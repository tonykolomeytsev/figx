use std::{
    collections::{BTreeSet, HashSet},
    path::PathBuf,
};

use crate::CanBeExtendedBy;

use super::VariantsDto;

#[derive(Default)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct ComposeProfileDto {
    pub remote_id: Option<String>,
    pub src_dir: Option<PathBuf>,
    pub package: Option<String>,
    pub kotlin_explicit_api: Option<bool>,
    pub extension_target: Option<String>,
    pub file_suppress_lint: Option<BTreeSet<String>>,
    pub color_mappings: Option<Vec<ColorMappingDto>>,
    pub preview: Option<ComposePreviewDto>,
    pub variants: Option<VariantsDto>,
    pub composable_get: Option<bool>,
}

impl CanBeExtendedBy<ComposeProfileDto> for ComposeProfileDto {
    fn extend(&self, another: &ComposeProfileDto) -> Self {
        Self {
            remote_id: another
                .remote_id
                .as_ref()
                .or(self.remote_id.as_ref())
                .cloned(),
            src_dir: another.src_dir.as_ref().or(self.src_dir.as_ref()).cloned(),
            package: another.package.as_ref().or(self.package.as_ref()).cloned(),
            kotlin_explicit_api: another.kotlin_explicit_api.or(self.kotlin_explicit_api),
            extension_target: another
                .extension_target
                .as_ref()
                .or(self.extension_target.as_ref())
                .cloned(),
            file_suppress_lint: another
                .file_suppress_lint
                .as_ref()
                .or(self.file_suppress_lint.as_ref())
                .cloned(),
            color_mappings: another
                .color_mappings
                .as_ref()
                .or(self.color_mappings.as_ref())
                .cloned(),
            preview: another.preview.as_ref().or(self.preview.as_ref()).cloned(),
            variants: match (another.variants.as_ref(), self.variants.as_ref()) {
                (Some(another), Some(this)) => Some(another.extend(this)),
                (Some(another), None) => Some(another.clone()),
                (None, Some(this)) => Some(this.clone()),
                _ => None,
            },
            composable_get: another.composable_get.or(self.composable_get),
        }
    }
}

pub(crate) struct ComposeProfileDtoContext<'a> {
    pub declared_remote_ids: &'a HashSet<String>,
    pub raster_only_remote_ids: &'a HashSet<String>,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct ColorMappingDto {
    pub from: String,
    pub to: String,
    pub imports: Vec<String>,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct ComposePreviewDto {
    pub imports: Vec<String>,
    pub code: String,
}

mod de {
    use super::*;
    use crate::ParseWithContext;
    use crate::parser::util::validate_remote_id;
    use toml_span::Deserialize;
    use toml_span::de_helpers::TableHelper;

    impl<'de> ParseWithContext<'de> for ComposeProfileDto {
        type Context = ComposeProfileDtoContext<'de>;

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let remote_id = th.optional_s::<String>("remote");
            let src_dir = th.optional::<String>("src_dir").map(PathBuf::from);
            let package = th.optional("package");
            let kotlin_explicit_api = th.optional("kotlin_explicit_api");
            let file_suppress_lint = th
                .optional::<Vec<String>>("file_suppress_lint")
                .map(|vec| vec.into_iter().collect::<BTreeSet<_>>());
            let extension_target = th.optional("extension_target");
            let color_mappings = th.optional("color_mappings");
            let preview = th.optional("preview");
            let variants = th.optional::<VariantsDto>("variants");
            let composable_get = th.optional("composable_get");
            th.finalize(None)?;
            // endregion: extract

            // region: validate
            let remote_id = validate_remote_id(
                remote_id,
                ctx.declared_remote_ids,
                Some(ctx.raster_only_remote_ids),
            )?;
            // endregion: validate

            Ok(Self {
                remote_id,
                src_dir,
                package,
                kotlin_explicit_api,
                file_suppress_lint,
                extension_target,
                color_mappings,
                preview,
                variants,
                composable_get,
            })
        }
    }

    impl<'de> Deserialize<'de> for ColorMappingDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            let mut th = TableHelper::new(value)?;
            let from = th.required("from")?;
            let to = th.required("to")?;
            let imports = th.optional("imports").unwrap_or_default();
            th.finalize(None)?;

            Ok(Self { from, to, imports })
        }
    }

    impl<'de> Deserialize<'de> for ComposePreviewDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            let mut th = TableHelper::new(value)?;
            let imports = th.optional("imports").unwrap_or_default();
            let code = th.required("code")?;
            th.finalize(None)?;

            Ok(Self { imports, code })
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {

    use super::*;
    use crate::{ParseWithContext, variant_dto};
    use ordermap::ordermap;
    use toml_span::Span;
    use unindent::unindent;

    #[test]
    fn ComposeProfileDto__valid_fully_defined_toml__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        remote = "figma"
        src_dir = "src/main/kotlin"
        package = "com.example"
        kotlin_explicit_api = true
        extension_target = "com.example.Icons"
        file_suppress_lint = ["MagicNumbers"]
        color_mappings = [{ from = "*", to = "Color.Black" }]
        preview.imports = ["com.example.Preview"]
        preview.code = "lorem ipsum dolor sit amet"
        composable_get = false
        variants.small = { output_name = "{base}Small", figma_name = "{base} / small", scale = 1.0 }
        variants.big = { output_name = "{base}Big", figma_name = "{base} / big", scale = 2.0 }
        variants.use = ["small", "big"]
        "#;
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let expected_dto = ComposeProfileDto {
            remote_id: Some("figma".to_string()),
            src_dir: Some(PathBuf::from("src/main/kotlin")),
            package: Some("com.example".to_string()),
            kotlin_explicit_api: Some(true),
            extension_target: Some("com.example.Icons".to_string()),
            file_suppress_lint: Some(["MagicNumbers".to_string()].into_iter().collect()),
            color_mappings: Some(vec![ColorMappingDto {
                from: "*".to_string(),
                to: "Color.Black".to_string(),
                imports: vec![],
            }]),
            preview: Some(ComposePreviewDto {
                imports: vec!["com.example.Preview".to_string()],
                code: "lorem ipsum dolor sit amet".to_string(),
            }),
            composable_get: Some(false),
            variants: Some(VariantsDto {
                all_variants: Some(ordermap! {
                    // alphabetic keys sorting because of BTreeMap under the hood of the toml parser
                    "big".to_string() => variant_dto! { "{base}Big" <- "{base} / big" (x 2.0) },
                    "small".to_string() => variant_dto! { "{base}Small" <- "{base} / small" (x 1.0) },
                }),
                use_variants: Some(vec!["small".to_string(), "big".to_string()]),
            }),
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let ctx = ComposeProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
            raster_only_remote_ids: &HashSet::new(),
        };
        let actual_dto = ComposeProfileDto::parse_with_ctx(&mut value, ctx).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn ComposeProfileDto__valid_empty_toml__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        "#;
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let expected_dto = ComposeProfileDto {
            remote_id: None,
            src_dir: None,
            package: None,
            kotlin_explicit_api: None,
            extension_target: None,
            file_suppress_lint: None,
            color_mappings: None,
            preview: None,
            composable_get: None,
            variants: None,
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let ctx = ComposeProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
            raster_only_remote_ids: &HashSet::new(),
        };
        let actual_dto = ComposeProfileDto::parse_with_ctx(&mut value, ctx).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn ComposeProfileDto__valid_invalid_remote__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                remote = "undeclared"
                package = "com.example"
            "#,
        );
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let err_spans = [Span::new(10, 20)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let ctx = ComposeProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
            raster_only_remote_ids: &HashSet::new(),
        };
        let actual_err = ComposeProfileDto::parse_with_ctx(&mut value, ctx).unwrap_err();

        // Then
        assert_eq!(err_spans.len(), actual_err.errors.len());
        for (expected_span, actual_err) in err_spans.into_iter().zip(actual_err.errors) {
            assert_eq!(expected_span, actual_err.span);
        }
    }

    #[test]
    fn ComposeProfileDto__valid_undeclared_key__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                remote = "figma"
                scale = 0.42
                dolor = 1234567
                output_dir = "images"
                lorem = "ipsum"
            "#,
        );
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let err_spans = [Span::new(30, 35), Span::new(68, 73)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let ctx = ComposeProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
            raster_only_remote_ids: &HashSet::new(),
        };
        let actual_err = ComposeProfileDto::parse_with_ctx(&mut value, ctx).unwrap_err();

        // Then
        for actual_err in actual_err.errors {
            if let toml_span::Error {
                kind: toml_span::ErrorKind::UnexpectedKeys { keys, .. },
                ..
            } = actual_err
            {
                for ((_, actual_span), expected_span) in keys.into_iter().zip(err_spans) {
                    assert_eq!(expected_span, actual_span);
                }
            }
        }
    }
}
