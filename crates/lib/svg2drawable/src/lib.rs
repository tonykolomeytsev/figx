use colorsys::ColorAlpha;
use lib_image_vector::{
    Cap, Color, Command, FillType, GroupNode, ImageVector, Join, LinearGradient, Node, PathNode,
    Point, RadialGradient,
};
use xmlwriter::Indent;

pub type Result<T> = std::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

const ATTRIBUTE_INDENT: Indent = Indent::Spaces(4);

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
        attributes_indent: ATTRIBUTE_INDENT,
        ..Default::default()
    };
    let mut w = xmlwriter::XmlWriter::new(Vec::<u8>::new(), opt);

    if options.xml_declaration {
        w.write_declaration()?;
    }

    let ImageVector {
        name: _,
        width,
        height,
        viewport_width,
        viewport_height,
        nodes,
    } = iv;

    w.start_element("vector")?;
    w.write_attribute(
        "xmlns:android",
        "http://schemas.android.com/apk/res/android",
    )?;

    if nodes.iter().any(has_gradients) {
        w.write_attribute("xmlns:aapt", "http://schemas.android.com/aapt")?;
    }

    w.write_attribute("android:height", &format!("{}dp", height))?;
    w.write_attribute("android:width", &format!("{}dp", width))?;
    w.write_attribute("android:viewportWidth", &format!("{}", viewport_width))?;
    w.write_attribute("android:viewportHeight", &format!("{}", viewport_height))?;

    for node in nodes {
        codegen_node(&mut w, node)?;
    }

    Ok(String::from_utf8(w.end_document()?)?)
}

fn codegen_node(w: &mut xmlwriter::XmlWriter<Vec<u8>>, node: Node) -> Result<()> {
    match node {
        Node::Path(path) => codegen_path_node(w, path),
        Node::Group(group) => codegen_group_node(w, group),
    }
}

fn codegen_group_node(w: &mut xmlwriter::XmlWriter<Vec<u8>>, group: GroupNode) -> Result<()> {
    let GroupNode {
        name,
        nodes,
        rotate,
        pivot,
        translation,
        scale,
        clip_path_data,
    } = group;
    w.start_element("group")?;
    if let Some(name) = name {
        w.write_attribute("android:name", &name)?;
    }
    if rotate != 0.0 {
        w.write_attribute("android:rotation", &format!("{}", rotate.to_degrees()))?;
    }
    if pivot.x != 0.0 {
        w.write_attribute("android:pivotX", &format!("{}", pivot.x))?;
    }
    if pivot.y != 0.0 {
        w.write_attribute("android:pivotY", &format!("{}", pivot.y))?;
    }
    if scale.x != 1.0 {
        w.write_attribute("android:scaleX", &format!("{}", scale.x))?;
    }
    if scale.y != 1.0 {
        w.write_attribute("android:scaleY", &format!("{}", scale.y))?;
    }
    if translation.x != 0.0 {
        w.write_attribute("android:translateX", &format!("{}", translation.x))?;
    }
    if translation.y != 0.0 {
        w.write_attribute("android:translateY", &format!("{}", translation.y))?;
    }

    if let Some(clip_path_data) = clip_path_data {
        w.start_element("clip-path")?;
        w.set_attribute_indent(xmlwriter::Indent::None);
        codegen_commands(w, &clip_path_data)?;
        w.set_attribute_indent(ATTRIBUTE_INDENT);
        w.end_element()?;
    }

    for node in nodes {
        codegen_node(w, node)?;
    }

    w.end_element()?;
    Ok(())
}

fn codegen_path_node(w: &mut xmlwriter::XmlWriter<Vec<u8>>, path: PathNode) -> Result<()> {
    let PathNode {
        fill_type,
        fill_color,
        commands,
        alpha,
        stroke,
    } = path;

    w.start_element("path")?;

    codegen_commands(w, &commands)?;
    if let Some(Color::SolidColor(rgb)) = &fill_color {
        w.write_attribute("android:fillColor", &hex_argb(rgb))?;
    }
    if let FillType::EvenOdd = fill_type {
        // non-default
        w.write_attribute("android:fillType", "evenOdd")?;
    }
    if alpha != 1.0 {
        w.write_attribute("android:fillAlpha", &format!("{}", alpha))?;
    }
    if let Some(Color::SolidColor(rgb)) = &stroke.color {
        w.write_attribute("android:strokeColor", &hex_argb(rgb))?;
    }
    match stroke.cap {
        Cap::Butt => (), // default
        Cap::Round => w.write_attribute("android:strokeLineCap", "round")?,
        Cap::Square => w.write_attribute("android:strokeLineCap", "square")?,
    }
    match stroke.join {
        Join::Bevel => (), // default
        Join::Miter => w.write_attribute("android:strokeLineJoin", "miter")?,
        Join::Round => w.write_attribute("android:strokeLineJoin", "round")?,
    }
    if stroke.width != 1.0 {
        w.write_attribute("android:strokeWidth", &format!("{}", stroke.width))?;
    }
    if stroke.alpha != 1.0 {
        w.write_attribute("android:strokeAlpha", &format!("{}", stroke.alpha))?;
    }
    if stroke.miter != 1.0 {
        w.write_attribute("android:strokeMiterLimit", &format!("{}", stroke.miter))?;
    }

    match &fill_color {
        Some(Color::LinearGradient(g)) => codegen_linear_gradient(w, &g, "android:fillColor")?,
        Some(Color::RadialGradient(g)) => codegen_radial_gradient(w, &g, "android:fillColor")?,
        _ => (),
    }
    match &stroke.color {
        Some(Color::LinearGradient(g)) => codegen_linear_gradient(w, &g, "android:strokeColor")?,
        Some(Color::RadialGradient(g)) => codegen_radial_gradient(w, &g, "android:strokeColor")?,
        _ => (),
    }

    w.end_element()?;
    Ok(())
}

fn codegen_commands(w: &mut xmlwriter::XmlWriter<Vec<u8>>, commands: &[Command]) -> Result<()> {
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
    w.write_attribute("android:pathData", &path_data)?;
    Ok(())
}

fn codegen_linear_gradient(
    w: &mut xmlwriter::XmlWriter<Vec<u8>>,
    g: &LinearGradient,
    attr_name: &str,
) -> Result<()> {
    w.start_element("aapt:attr")?;
    w.set_attribute_indent(Indent::None);
    w.write_attribute("name", attr_name)?;
    w.set_attribute_indent(ATTRIBUTE_INDENT);
    w.start_element("gradient")?;
    w.write_attribute("android:startX", &format!("{}", g.start_x))?;
    w.write_attribute("android:startY", &format!("{}", g.start_y))?;
    w.write_attribute("android:endX", &format!("{}", g.end_x))?;
    w.write_attribute("android:endY", &format!("{}", g.end_y))?;
    w.write_attribute("android:type", "linear")?;

    for stop in g.stops.iter() {
        w.start_element("item")?;
        w.set_attribute_indent(Indent::None);
        w.write_attribute("android:offset", &format!("{}", stop.offset))?;
        w.write_attribute("android:color", &hex_argb(&stop.color))?;
        w.set_attribute_indent(ATTRIBUTE_INDENT);
        w.end_element()?;
    }

    w.end_element()?;
    w.end_element()?;
    Ok(())
}

fn codegen_radial_gradient(
    w: &mut xmlwriter::XmlWriter<Vec<u8>>,
    g: &RadialGradient,
    attr_name: &str,
) -> Result<()> {
    w.start_element("aapt:attr")?;
    w.set_attribute_indent(Indent::None);
    w.write_attribute("name", attr_name)?;
    w.set_attribute_indent(ATTRIBUTE_INDENT);
    w.start_element("gradient")?;
    w.write_attribute("android:gradientRadius", &format!("{}", g.gradient_radius))?;
    w.write_attribute("android:centerX", &format!("{}", g.center_x))?;
    w.write_attribute("android:centerY", &format!("{}", g.center_y))?;
    w.write_attribute("android:type", "radial")?;

    for stop in g.stops.iter() {
        w.start_element("item")?;
        w.set_attribute_indent(Indent::None);
        w.write_attribute("android:offset", &format!("{}", stop.offset))?;
        w.write_attribute("android:color", &hex_argb(&stop.color))?;
        w.set_attribute_indent(ATTRIBUTE_INDENT);
        w.end_element()?;
    }

    w.end_element()?;
    w.end_element()?;
    Ok(())
}

fn has_gradients(node: &Node) -> bool {
    match node {
        Node::Path(p) => match (&p.fill_color, &p.stroke.color) {
            (Some(Color::LinearGradient(_)), _)
            | (Some(Color::RadialGradient(_)), _)
            | (_, Some(Color::LinearGradient(_)))
            | (_, Some(Color::RadialGradient(_))) => true,
            _ => false,
        },
        Node::Group(g) => g.nodes.iter().any(has_gradients),
    }
}

fn hex_argb(color: &colorsys::Rgb) -> String {
    let a = (color.alpha() * 255.0).round() as u8;
    let r = (color.red().round()) as u8;
    let g = (color.green().round()) as u8;
    let b = (color.blue().round()) as u8;
    if a == 255 {
        format!("#{r:02X}{g:02X}{b:02X}").to_ascii_uppercase()
    } else {
        format!("#{a:02X}{r:02X}{g:02X}{b:02X}").to_ascii_uppercase()
    }
}
