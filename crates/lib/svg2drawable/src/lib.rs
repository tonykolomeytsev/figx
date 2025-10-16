use lib_image_vector::{
    Cap, Color, Command, FillType, GroupNode, ImageVector, Join, Node, PathNode, Point,
};

pub type Result<T> = std::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct SvgToDrawableOptions {
    /// Add `<?xml version="1.0" encoding="UTF-8"?>` declaration to the XML output
    pub xml_declaration: bool,
}

pub fn transform_svg_to_drawable(svg: &[u8], options: SvgToDrawableOptions) -> Result<Vec<u8>> {
    let tree = usvg::Tree::from_data(svg, &Default::default())?;
    let image_vector: ImageVector = tree.try_into()?;
    let output = codegen_xml(image_vector, options)?;
    Ok(output.into_bytes())
}

fn codegen_xml(iv: ImageVector, options: SvgToDrawableOptions) -> Result<String> {
    let opt = xmlwriter::Options {
        use_single_quote: false,
        indent: xmlwriter::Indent::Spaces(2),
        attributes_indent: xmlwriter::Indent::Spaces(4),
    };
    let mut w = xmlwriter::XmlWriter::new(opt);

    if options.xml_declaration {
        w.write_declaration();
    }

    let ImageVector {
        name: _,
        width,
        height,
        viewport_width,
        viewport_height,
        nodes,
    } = iv;

    w.start_element("vector");
    w.write_attribute(
        "xmlns:android",
        "http://schemas.android.com/apk/res/android",
    );
    w.write_attribute("android:height", &format!("{}dp", height));
    w.write_attribute("android:width", &format!("{}dp", width));
    w.write_attribute("android:viewportWidth", &format!("{}", viewport_width));
    w.write_attribute("android:viewportHeight", &format!("{}", viewport_height));

    for node in nodes {
        codegen_node(&mut w, node)?;
    }

    Ok(w.end_document())
}

fn codegen_node(w: &mut xmlwriter::XmlWriter, node: Node) -> Result<()> {
    match node {
        Node::Path(path) => codegen_path_node(w, path),
        Node::Group(group) => codegen_group_node(w, group),
    }
}

fn codegen_group_node(w: &mut xmlwriter::XmlWriter, group: GroupNode) -> Result<()> {
    let GroupNode {
        name,
        nodes,
        rotate,
        pivot,
        translation,
        scale,
    } = group;
    w.start_element("group");
    if let Some(name) = name {
        w.write_attribute("android:name", &name);
    }
    if rotate != 0.0 {
        w.write_attribute("android:rotation", &format!("{rotate}f"));
        w.write_attribute("android:pivotX", &format!("{}", pivot.x));
        w.write_attribute("android:pivotY", &format!("{}", pivot.y));
    }
    if scale.x != 1.0 || scale.y != 1.0 {
        w.write_attribute("android:scaleX", &format!("{}", scale.x));
        w.write_attribute("android:scaleY", &format!("{}", scale.y));
    }
    if translation.x != 0.0 || translation.y != 0.0 {
        w.write_attribute("android:translationX", &format!("{}", translation.x));
        w.write_attribute("android:translationY", &format!("{}", translation.y));
    }

    for node in nodes {
        codegen_node(w, node)?;
    }

    w.end_element();
    Ok(())
}

fn codegen_path_node(w: &mut xmlwriter::XmlWriter, path: PathNode) -> Result<()> {
    let PathNode {
        fill_type,
        fill_color,
        commands,
        alpha,
        stroke,
    } = path;

    w.start_element("path");

    codegen_commands(w, &commands);
    if let Some(Color::SolidColor(rgb)) = fill_color {
        w.write_attribute("android:fillColor", &format!("{}", rgb.to_hex_string()));
    }
    if let FillType::EvenOdd = fill_type {
        // non-default
        w.write_attribute("android:fillType", "evenOdd");
    }
    if alpha != 1.0 {
        w.write_attribute("android:fillAlpha", &format!("{}", alpha));
    }
    if let Some(Color::SolidColor(rgb)) = stroke.color {
        w.write_attribute("android:strokeColor", &format!("{}", rgb.to_hex_string()));
    }
    match stroke.cap {
        Cap::Butt => (), // default
        Cap::Round => w.write_attribute("android:strokeLineCap", "round"),
        Cap::Square => w.write_attribute("android:strokeLineCap", "square"),
    }
    match stroke.join {
        Join::Bevel => (), // default
        Join::Miter => w.write_attribute("android:strokeLineJoin", "miter"),
        Join::Round => w.write_attribute("android:strokeLineJoin", "round"),
    }
    if stroke.width != 1.0 {
        w.write_attribute("android:strokeWidth", &format!("{}", stroke.width));
    }
    if stroke.alpha != 1.0 {
        w.write_attribute("android:strokeAlpha", &format!("{}", stroke.alpha));
    }
    if stroke.miter != 1.0 {
        w.write_attribute("android:strokeMiterLimit", &format!("{}", stroke.miter));
    }

    w.end_element();
    Ok(())
}

fn codegen_commands(w: &mut xmlwriter::XmlWriter, commands: &[Command]) {
    let mut path_data = String::new();
    for command in commands {
        match command {
            Command::MoveTo(Point { x, y }) => {
                path_data.push_str(&format!("M{},{}", x, y));
            }
            Command::LineTo(Point { x, y }) => {
                path_data.push_str(&format!("L{},{}", x, y));
            }
            Command::CurveTo(
                Point { x: x1, y: y1 },
                Point { x: x2, y: y2 },
                Point { x: x3, y: y3 },
            ) => {
                path_data.push_str(&format!("C{},{} {},{} {},{}", x1, y1, x2, y2, x3, y3));
            }
            Command::QuadraticBezierTo(Point { x: x1, y: y1 }, Point { x: x2, y: y2 }) => {
                path_data.push_str(&format!("Q{},{} {},{}", x1, y1, x2, y2));
            }
            Command::Close => {
                path_data.push('Z');
            }
        }
    }
    w.write_attribute("android:pathData", &path_data);
}
