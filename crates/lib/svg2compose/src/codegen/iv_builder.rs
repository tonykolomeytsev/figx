use std::fmt::Display;

use crate::{
    ColorMapping,
    kotlin::{CodeBlock, Touch},
};
use colorsys::Rgb;
use lib_image_vector::{
    Cap, Color, Command, FillType, GroupNode, ImageVector, Join, Node, PathNode, Point,
};
use log::debug;

type Result<T> = std::result::Result<T, IVBuilderError>;
#[derive(Debug)]
pub enum IVBuilderError {
    InvalidMappingColor(colorsys::ParseError),
    UnsupportedFillType(String),
}

// region: Error boilerplate

impl std::error::Error for IVBuilderError {}
impl Display for IVBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMappingColor(e) => write!(f, "invalid mapping color: {e}"),
            Self::UnsupportedFillType(t) => write!(f, "unsupported fill type: {t}"),
        }
    }
}

// endregion: Error boilerplate

pub fn codegen_iv_builder(iv: ImageVector, color_mappings: &[ColorMapping]) -> Result<CodeBlock> {
    let ImageVector {
        name,
        width,
        height,
        viewport_width,
        viewport_height,
        nodes,
    } = iv;
    let code = CodeBlock::builder()
        .add_statement("ImageVector.Builder(")
        .indent()
        .add_statement(format!("name = \"{name}\","))
        .add_statement(format!("defaultWidth = {width}.dp,"))
        .add_statement(format!("defaultHeight = {height}.dp,"))
        .add_statement(format!("viewportWidth = {viewport_width}f,"))
        .add_statement(format!("viewportHeight = {viewport_height}f,"))
        .unindent()
        .begin_control_flow(").apply {")
        .add_code_blocks(
            nodes
                .into_iter()
                .map(|n| codegen_node(n, color_mappings))
                .collect::<Result<Vec<_>>>()?,
        )
        .end_control_flow()
        .no_new_line()
        .add_statement(".build()")
        .require_imports(&[
            "androidx.compose.ui.unit.dp",
            "androidx.compose.ui.graphics.vector.ImageVector",
        ])
        .build();
    Ok(code)
}

fn codegen_node(n: Node, color_mappings: &[ColorMapping]) -> Result<CodeBlock> {
    match n {
        Node::Path(path) => codegen_path_node(path, color_mappings),
        Node::Group(group) => codegen_group_node(group, color_mappings),
    }
}

fn codegen_group_node(n: GroupNode, color_mappings: &[ColorMapping]) -> Result<CodeBlock> {
    let GroupNode {
        name,
        nodes,
        rotate,
        pivot,
        translation,
        scale,
    } = n;
    let code = CodeBlock::builder()
        .add_statement("group(")
        .indent()
        .touch(|it| match name {
            Some(name) => it.add_statement(format!("name = \"{name}\",")),
            None => it,
        })
        .add_statement(format!("rotate = {rotate}f,"))
        .add_statement(format!("pivotX = {}f,", pivot.x))
        .add_statement(format!("pivotY = {}f,", pivot.y))
        .add_statement(format!("scaleX = {}f,", scale.x))
        .add_statement(format!("scaleY = {}f,", scale.y))
        .add_statement(format!("translationX = {}f,", translation.x))
        .add_statement(format!("translationY = {}f,", translation.y))
        .add_statement("clipPathData = emptyList(),")
        .unindent()
        .begin_control_flow(") {")
        .add_code_blocks(
            nodes
                .into_iter()
                .map(|n| codegen_node(n, color_mappings))
                .collect::<Result<Vec<_>>>()?,
        )
        .end_control_flow()
        .require_import("androidx.compose.ui.graphics.vector.group")
        .build();
    Ok(code)
}

fn codegen_path_node(n: PathNode, color_mappings: &[ColorMapping]) -> Result<CodeBlock> {
    let PathNode {
        fill_type,
        fill_color,
        commands,
        alpha,
        stroke,
    } = n;
    let fill_color = match fill_color {
        Some(c) => Some(mapped_color(c, color_mappings)?),
        None => None,
    };
    // TODO: support gradients
    let (stroke_color, stroke_color_imports) = match stroke.color {
        Some(c) => mapped_color(c, color_mappings)?,
        None => ("null".to_string(), Vec::new()),
    };
    let stroke_cap_str = match stroke.cap {
        Cap::Butt => "StrokeCap.Butt",
        Cap::Square => "StrokeCap.Square",
        Cap::Round => "StrokeCap.Round",
    };
    let stroke_join_str = match stroke.join {
        Join::Bevel => "StrokeJoin.Bevel",
        Join::Miter => "StrokeJoin.Miter",
        Join::Round => "StrokeJoin.Round",
    };
    let fill_type_str = match fill_type {
        FillType::NonZero => "PathFillType.NonZero",
        FillType::EvenOdd => "PathFillType.EvenOdd",
    };
    let code = CodeBlock::builder()
        .add_statement("path(")
        .indent()
        .touch(|it| match fill_color {
            Some((color, imports)) => it
                .add_statement(format!("fill = {color},"))
                .require_imports(&imports),
            None => it,
        })
        .touch(|it| match alpha {
            1.0f32 => it,
            alpha => it.add_statement(format!("fillAlpha = {alpha}f,")),
        })
        .add_statement(format!("stroke = {stroke_color},"))
        .require_imports(&stroke_color_imports)
        .touch(|it| match stroke.alpha {
            1.0f32 => it,
            alpha => it.add_statement(format!("strokeAlpha = {alpha}f,")),
        })
        .touch(|it| match stroke.width {
            0.0f32 => it,
            width => it.add_statement(format!("strokeLineWidth = {width}f,")),
        })
        .touch(|it| match stroke.cap {
            Cap::Butt => it,
            _ => it
                .add_statement(format!("strokeLineCap = {stroke_cap_str},"))
                .require_import("androidx.compose.ui.graphics.StrokeCap"),
        })
        .touch(|it| match stroke.join {
            Join::Miter => it,
            _ => it
                .add_statement(format!("strokeLineJoin = {stroke_join_str},"))
                .require_import("androidx.compose.ui.graphics.StrokeJoin"),
        })
        .touch(|it| match stroke.miter {
            4.0f32 => it,
            miter => it.add_statement(format!("strokeLineMiter = {miter}f,")),
        })
        .touch(|it| match fill_type {
            FillType::NonZero => it,
            FillType::EvenOdd => it
                .add_statement(format!("pathFillType = {fill_type_str}"))
                .require_import("androidx.compose.ui.graphics.PathFillType"),
        })
        .unindent()
        .begin_control_flow(") {")
        .add_code_blocks(commands.into_iter().map(Into::into).collect())
        .end_control_flow()
        .require_imports(&[
            "androidx.compose.ui.graphics.Color",
            "androidx.compose.ui.graphics.SolidColor",
            "androidx.compose.ui.graphics.vector.path",
        ])
        .build();
    Ok(code)
}

impl From<Command> for CodeBlock {
    fn from(value: Command) -> Self {
        Self::builder()
            .add_statement(match value {
                Command::Close => "close()".to_string(),
                Command::CurveTo(
                    Point { x: x1, y: y1 },
                    Point { x: x2, y: y2 },
                    Point { x: x3, y: y3 },
                ) => format!("curveTo({x1}f, {y1}f, {x2}f, {y2}f, {x3}f, {y3}f)"),
                Command::QuadraticBezierTo(Point { x: x1, y: y1 }, Point { x: x2, y: y2 }) => {
                    format!("quadTo({x1}f, {y1}f, {x2}f, {y2}f)")
                }
                Command::LineTo(Point { x, y }) => format!("lineTo({x}f, {y}f)"),
                Command::MoveTo(Point { x, y }) => format!("moveTo({x}f, {y}f)"),
            })
            .build()
    }
}

fn mapped_color(c: Color, color_mappings: &[ColorMapping]) -> Result<(String, Vec<String>)> {
    let rgb = match c {
        Color::SolidColor(c) => c,
        Color::LinearGradient(_) => {
            return Err(IVBuilderError::UnsupportedFillType(
                "linear-gradient".to_string(),
            ));
        }
        Color::RadialGradient(_) => {
            return Err(IVBuilderError::UnsupportedFillType(
                "radial-gradient".to_string(),
            ));
        }
    };
    for mapping in color_mappings {
        if mapping.from == "*"
            || rgb
                == Rgb::from_hex_str(&mapping.from)
                    .map_err(|e| IVBuilderError::InvalidMappingColor(e))?
        {
            debug!(target: "Svg2Compose", "Found color mapping match: {} -> {}", mapping.from, mapping.to);
            return Ok((
                format!("SolidColor({})", mapping.to.to_owned()),
                mapping.imports.to_owned(),
            ));
        }
    }
    Ok((
        format!(
            "SolidColor(Color(0xFF{:02X}{:02X}{:02X}))",
            rgb.red() as u8,
            rgb.green() as u8,
            rgb.blue() as u8
        ),
        vec![
            "androidx.compose.ui.graphics.Color".to_owned(),
            "androidx.compose.ui.graphics.SolidColor".to_owned(),
        ],
    ))
}
