use crate::SvgToComposeOptions;
use crate::image_vector::*;
use crate::kotlin::*;

pub struct BackingFieldComposableSpec {
    pub options: SvgToComposeOptions,
    pub image_vector: ImageVector,
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

impl From<PathNode> for CodeBlock {
    fn from(value: PathNode) -> Self {
        let PathNode {
            fill_type,
            fill_color,
            commands,
            alpha,
            stroke,
        } = value;
        // TODO: support gradients
        let stroke_color = match stroke.color {
            Some(c) => c.as_solid_color(),
            None => "null".to_string(),
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
        Self::builder()
            .add_statement("path(")
            .indent()
            .touch(|it| match fill_color {
                Some(c) => it.add_statement(format!("fill = {},", c.as_solid_color())),
                None => it,
            })
            .touch(|it| match alpha {
                1.0f32 => it,
                alpha => it.add_statement(format!("fillAlpha = {alpha}f,")),
            })
            .add_statement(format!("stroke = {stroke_color},"))
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
            .build()
    }
}

impl From<GroupNode> for CodeBlock {
    fn from(value: GroupNode) -> Self {
        let GroupNode {
            name,
            nodes,
            rotate,
            pivot,
            translation,
            scale,
        } = value;
        let name = match name {
            Some(name) => format!("\"{name}\""),
            None => "null".to_string(),
        };
        Self::builder()
            .add_statement("group(")
            .indent()
            .add_statement(format!("name = {name},"))
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
            .add_code_blocks(nodes.into_iter().map(Into::into).collect())
            .end_control_flow()
            .require_import("androidx.compose.ui.graphics.vector.group")
            .build()
    }
}

impl From<Node> for CodeBlock {
    fn from(value: Node) -> Self {
        match value {
            Node::Path(path) => path.into(),
            Node::Group(group) => group.into(),
        }
    }
}

impl From<ImageVector> for CodeBlock {
    fn from(value: ImageVector) -> Self {
        let ImageVector {
            name,
            width,
            height,
            viewport_width,
            viewport_height,
            nodes,
        } = value;
        Self::builder()
            .add_statement("ImageVector.Builder(")
            .indent()
            .add_statement(format!("name = \"{name}\","))
            .add_statement(format!("defaultWidth = {width}.dp,"))
            .add_statement(format!("defaultHeight = {height}.dp,"))
            .add_statement(format!("viewportWidth = {viewport_width}f,"))
            .add_statement(format!("viewportHeight = {viewport_height}f,"))
            .unindent()
            .begin_control_flow(").apply {")
            .add_code_blocks(nodes.into_iter().map(Into::into).collect())
            .end_control_flow()
            .add_statement(".build()")
            .require_imports(&[
                "androidx.compose.ui.unit.dp",
                "androidx.compose.ui.graphics.vector.ImageVector",
            ])
            .build()
    }
}

impl From<BackingFieldComposableSpec> for FileSpec {
    fn from(value: BackingFieldComposableSpec) -> Self {
        let BackingFieldComposableSpec {
            options,
            image_vector,
        } = value;
        let SvgToComposeOptions {
            image_name,
            package,
            kotlin_explicit_api,
            extension_target,
            file_suppress_lint,
            color_mappings: _,
            preview,
        } = options;

        // region: determine extension target
        let (public_property_name, additional_import) = match &extension_target {
            Some(fq_name) => {
                if let Some((_, simple_name)) = fq_name.rsplit_once(".") {
                    (format!("{simple_name}.{image_name}"), Some(fq_name))
                } else {
                    (format!("{fq_name}.{image_name}"), None)
                }
            }
            None => (image_name.to_owned(), None),
        };
        // endregion: determine extension target

        let public_property = PropertySpec::builder(&public_property_name, "ImageVector")
            .require_import("androidx.compose.ui.graphics.vector.ImageVector")
            .touch(|it| match additional_import {
                Some(import) => it.require_import(import),
                None => it,
            })
            .touch(|it| match kotlin_explicit_api {
                true => it.add_modifier("public"),
                false => it,
            })
            .getter(
                CodeBlock::builder()
                    .begin_control_flow("get()")
                    .begin_control_flow(format!("if (_{image_name} != null)"))
                    .add_statement(format!("return _{image_name}!!"))
                    .end_control_flow()
                    .add_code_block(
                        CodeBlock::builder()
                            .add_statement(format!("_{image_name} = "))
                            .no_new_line()
                            .add_code_block(image_vector.into())
                            .build(),
                    )
                    .add_statement(format!("return _{image_name}!!"))
                    .end_control_flow()
                    .build(),
            )
            .build();

        let backing_field = PropertySpec::builder(format!("_{image_name}"), "ImageVector?")
            .require_import("androidx.compose.ui.graphics.vector.ImageVector")
            .initializer(CodeBlock::builder().add_statement("null").build())
            .add_annotation(r#"@Suppress("ObjectPropertyName")"#)
            .add_modifier("private")
            .mutable()
            .build();

        let preview_fun = if let Some(preview) = preview {
            let code = preview.code.replace("{name}", &image_name);
            CodeBlock::builder()
                .require_imports(&preview.imports)
                .add_statement(code)
                .build()
        } else {
            CodeBlock::builder()
                .add_statement("@Preview(showBackground = true)")
                .add_statement("@Composable")
                .begin_control_flow(format!("private fun {image_name}Preview() {{"))
                .add_statement("Icon(")
                .indent()
                .add_statement(format!("imageVector = {public_property_name},"))
                .add_statement("contentDescription = null,")
                .unindent()
                .add_statement(")")
                .end_control_flow()
                .require_imports(&[
                    "androidx.compose.material3.Icon",
                    "androidx.compose.runtime.Composable",
                    "androidx.compose.ui.tooling.preview.Preview",
                ])
                .build()
        };

        Self::builder(package)
            .add_suppressions(file_suppress_lint)
            .add_member(public_property.into())
            .add_member(backing_field.into())
            .add_member(preview_fun)
            .build()
    }
}
