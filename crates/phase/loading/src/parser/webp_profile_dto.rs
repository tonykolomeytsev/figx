use std::{collections::HashSet, path::PathBuf};

use crate::CanBeExtendedBy;

use super::VariantsDto;

#[derive(Default)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct WebpProfileDto {
    pub remote_id: Option<String>,
    pub scale: Option<f32>,
    pub quality: Option<f32>,
    pub output_dir: Option<PathBuf>,
    pub variants: Option<VariantsDto>,
}

impl CanBeExtendedBy<Self> for WebpProfileDto {
    fn extend(&self, another: &Self) -> Self {
        Self {
            remote_id: another
                .remote_id
                .as_ref()
                .or(self.remote_id.as_ref())
                .cloned(),
            scale: another.scale.or(self.scale),
            quality: another.quality.or(self.quality),
            output_dir: another
                .output_dir
                .as_ref()
                .or(self.output_dir.as_ref())
                .cloned(),
            variants: another
                .variants
                .as_ref()
                .or(self.variants.as_ref())
                .cloned(),
        }
    }
}

pub(crate) struct WebpProfileDtoContext<'a> {
    pub declared_remote_ids: &'a HashSet<String>,
}

mod de {
    use super::*;
    use crate::parser::de::parse_variants;
    use crate::ParseWithContext;
    use crate::parser::util::{validate_figma_scale, validate_remote_id, validate_webp_quality};
    use toml_span::de_helpers::TableHelper;

    impl<'de> ParseWithContext<'de> for WebpProfileDto {
        type Context = WebpProfileDtoContext<'de>;

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let remote_id = th.optional_s::<String>("remote");
            let scale = th.optional_s::<f32>("scale");
            let quality = th.optional_s::<f32>("quality");
            let output_dir = th.optional::<String>("output_dir").map(PathBuf::from);
            let variants = parse_variants(&mut th)?;
            th.finalize(None)?;
            // endregion: extract

            // region: validate
            let remote_id = validate_remote_id(remote_id, ctx.declared_remote_ids)?;
            let scale = validate_figma_scale(scale)?;
            let quality = validate_webp_quality(quality)?;
            // endregion: validate

            Ok(Self {
                remote_id,
                scale,
                quality,
                output_dir,
                variants,
            })
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {

    use super::*;
    use crate::ParseWithContext;
    use toml_span::Span;
    use unindent::unindent;

    #[test]
    fn WebpProfileDto__valid_fully_defined_toml__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        remote = "figma"
        scale = 0.42
        quality = 100,
        output_dir = "images"
        "#;
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let expected_dto = WebpProfileDto {
            remote_id: Some("figma".to_string()),
            scale: Some(0.42),
            quality: Some(100.0),
            output_dir: Some(PathBuf::from("images")),
            variants: None,
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let ctx = WebpProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_dto = WebpProfileDto::parse_with_ctx(&mut value, ctx).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn WebpProfileDto__valid_partially_defined_toml_v1__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        remote = "figma"
        output_dir = "images"
        "#;
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let expected_dto = WebpProfileDto {
            remote_id: Some("figma".to_string()),
            scale: None,
            quality: None,
            output_dir: Some(PathBuf::from("images")),
            variants: None,
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let ctx = WebpProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_dto = WebpProfileDto::parse_with_ctx(&mut value, ctx).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn WebpProfileDto__valid_partially_defined_toml_v2__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        remote = "figma"
        "#;
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let expected_dto = WebpProfileDto {
            remote_id: Some("figma".to_string()),
            scale: None,
            quality: None,
            output_dir: None,
            variants: None,
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let ctx = WebpProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_dto = WebpProfileDto::parse_with_ctx(&mut value, ctx).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn WebpProfileDto__valid_partially_defined_toml_v3__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        "#;
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let expected_dto = WebpProfileDto {
            remote_id: None,
            scale: None,
            quality: None,
            output_dir: None,
            variants: None,
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let ctx = WebpProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_dto = WebpProfileDto::parse_with_ctx(&mut value, ctx).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn WebpProfileDto__valid_invalid_remote__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                remote = 42
                scale = "0.42"
                output_dir = true
            "#,
        );
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let err_spans = [Span::new(9, 11), Span::new(21, 25), Span::new(40, 44)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let ctx = WebpProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_err = WebpProfileDto::parse_with_ctx(&mut value, ctx).unwrap_err();

        // Then
        for (expected_span, actual_err) in err_spans.into_iter().zip(actual_err.errors) {
            assert_eq!(expected_span, actual_err.span);
        }
    }

    #[test]
    fn WebpProfileDto__valid_undeclared_key__EXPECT__error_with_correct_span() {
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
        let ctx = WebpProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_err = WebpProfileDto::parse_with_ctx(&mut value, ctx).unwrap_err();

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
