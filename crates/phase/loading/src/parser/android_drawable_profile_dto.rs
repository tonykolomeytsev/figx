use crate::{CanBeExtendedBy, SingleNamePattern};
use std::{collections::HashSet, path::PathBuf};

#[derive(Default)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct AndroidDrawableProfileDto {
    pub remote_id: Option<String>,
    pub android_res_dir: Option<PathBuf>,
    pub night: Option<SingleNamePattern>,
    pub auto_mirrored: Option<bool>,
}

impl CanBeExtendedBy<Self> for AndroidDrawableProfileDto {
    fn extend(&self, another: &Self) -> Self {
        Self {
            remote_id: another
                .remote_id
                .as_ref()
                .or(self.remote_id.as_ref())
                .cloned(),
            android_res_dir: another
                .android_res_dir
                .as_ref()
                .or(self.android_res_dir.as_ref())
                .cloned(),
            night: another.night.as_ref().or(self.night.as_ref()).cloned(),
            auto_mirrored: another.auto_mirrored.or(self.auto_mirrored),
        }
    }
}

pub(crate) struct AndroidDrawableProfileDtoContext<'a> {
    pub declared_remote_ids: &'a HashSet<String>,
}

mod de {
    use super::*;
    use crate::ParseWithContext;
    use crate::parser::util::validate_remote_id;
    use toml_span::de_helpers::TableHelper;

    impl<'de> ParseWithContext<'de> for AndroidDrawableProfileDto {
        type Context = AndroidDrawableProfileDtoContext<'de>;

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let remote_id = th.optional_s::<String>("remote");
            let android_res_dir = th.optional::<String>("android_res_dir").map(PathBuf::from);
            let night = th.optional("night");
            let auto_mirrored = th.optional("auto_mirrored");
            th.finalize(None)?;
            // endregion: extract

            // region: validate
            let remote_id = validate_remote_id(remote_id, ctx.declared_remote_ids)?;
            // endregion: validate

            Ok(Self {
                remote_id,
                android_res_dir,
                night,
                auto_mirrored,
            })
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {

    use super::*;
    use crate::{ParseWithContext, SingleNamePattern};
    use toml_span::Span;
    use unindent::unindent;

    #[test]
    fn AndroidDrawableProfileDto__valid_fully_defined_toml__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        remote = "figma"
        android_res_dir = "src/main/res"
        night = "{base} / dark"
        auto_mirrored = false
        "#;
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let expected_dto = AndroidDrawableProfileDto {
            remote_id: Some("figma".to_string()),
            android_res_dir: Some(PathBuf::from("src/main/res")),
            night: Some(SingleNamePattern("{base} / dark".to_string())),
            auto_mirrored: Some(false),
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let ctx = AndroidDrawableProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_dto = AndroidDrawableProfileDto::parse_with_ctx(&mut value, ctx).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn AndroidDrawableProfileDto__valid_empty_toml__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        "#;
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let expected_dto = AndroidDrawableProfileDto {
            remote_id: None,
            android_res_dir: None,
            night: None,
            auto_mirrored: None,
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let ctx = AndroidDrawableProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_dto = AndroidDrawableProfileDto::parse_with_ctx(&mut value, ctx).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn AndroidDrawableProfileDto__valid_invalid_remote__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                remote = "undeclared"
                quality = 75
            "#,
        );
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let err_spans = [Span::new(0, 35)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let ctx = AndroidDrawableProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_err = AndroidDrawableProfileDto::parse_with_ctx(&mut value, ctx).unwrap_err();

        // Then
        assert_eq!(err_spans.len(), actual_err.errors.len());
        for (expected_span, actual_err) in err_spans.into_iter().zip(actual_err.errors) {
            assert_eq!(expected_span, actual_err.span);
        }
    }

    #[test]
    fn AndroidDrawableProfileDto__valid_undeclared_key__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                remote = "figma"
                dolor = 1234567
                lorem = "ipsum"
            "#,
        );
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let err_spans = [Span::new(17, 22), Span::new(33, 38)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let ctx = AndroidDrawableProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_err = AndroidDrawableProfileDto::parse_with_ctx(&mut value, ctx).unwrap_err();

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
