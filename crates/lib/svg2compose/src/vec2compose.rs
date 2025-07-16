use crate::SvgToComposeOptions;
use crate::kotlin::*;

pub struct BackingFieldComposableSpec {
    pub options: SvgToComposeOptions,
    pub iv_code_block: CodeBlock,
}

impl From<BackingFieldComposableSpec> for FileSpec {
    fn from(value: BackingFieldComposableSpec) -> Self {
        let BackingFieldComposableSpec {
            options,
            iv_code_block,
        } = value;
        let SvgToComposeOptions {
            image_name,
            package,
            kotlin_explicit_api,
            extension_target,
            file_suppress_lint,
            color_mappings: _,
            preview,
            composable_get,
        } = options;

        let backing_field_name = uncapitalize(&image_name);

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
                    .touch(|it| {
                        if composable_get {
                            it.begin_control_flow("@Composable get()")
                                .require_import("androidx.compose.runtime.Composable")
                        } else {
                            it.begin_control_flow("get()")
                        }
                    })
                    .begin_control_flow(format!("if (_{backing_field_name} != null)"))
                    .add_statement(format!("return _{backing_field_name}!!"))
                    .end_control_flow()
                    .add_code_block(
                        CodeBlock::builder()
                            .add_statement(format!("_{backing_field_name} = "))
                            .no_new_line()
                            .add_code_block(iv_code_block)
                            .build(),
                    )
                    .add_statement(format!("return _{backing_field_name}!!"))
                    .end_control_flow()
                    .build(),
            )
            .build();

        let backing_field = PropertySpec::builder(format!("_{backing_field_name}"), "ImageVector?")
            .require_import("androidx.compose.ui.graphics.vector.ImageVector")
            .initializer(CodeBlock::builder().add_statement("null").build())
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

fn uncapitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().chain(c).collect(),
    }
}
