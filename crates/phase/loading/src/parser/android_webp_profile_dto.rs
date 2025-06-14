use std::{
    collections::{BTreeSet, HashSet},
    path::PathBuf,
};

use crate::{CanBeExtendedBy, SingleNamePattern, WebpQuality};

#[derive(Default)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct AndroidWebpProfileDto {
    pub remote_id: Option<String>,
    pub android_res_dir: Option<PathBuf>,
    pub quality: Option<WebpQuality>,
    pub densities: Option<BTreeSet<AndroidDensityDto>>,
    pub night: Option<SingleNamePattern>,
    pub legacy_loader: Option<bool>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[cfg_attr(test, derive(Debug))]
pub(crate) enum AndroidDensityDto {
    LDPI,
    MDPI,
    HDPI,
    XHDPI,
    XXHDPI,
    XXXHDPI,
}

impl CanBeExtendedBy<Self> for AndroidWebpProfileDto {
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
            quality: another.quality.or(self.quality),
            densities: another
                .densities
                .as_ref()
                .or(self.densities.as_ref())
                .cloned(),
            night: another.night.as_ref().or(self.night.as_ref()).cloned(),
            legacy_loader: another.legacy_loader.or(self.legacy_loader),
        }
    }
}

pub(crate) struct AndroidWebpProfileDtoContext<'a> {
    pub declared_remote_ids: &'a HashSet<String>,
}

mod de {
    use super::*;
    use crate::parser::util::validate_remote_id;
    use crate::{ParseWithContext, WebpQuality};
    use toml_span::Deserialize;
    use toml_span::de_helpers::{TableHelper, expected};

    impl<'de> ParseWithContext<'de> for AndroidWebpProfileDto {
        type Context = AndroidWebpProfileDtoContext<'de>;

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let remote_id = th.optional_s::<String>("remote");
            let android_res_dir = th.optional::<String>("android_res_dir").map(PathBuf::from);
            let quality = th.optional::<WebpQuality>("quality");
            let densities = th
                .optional::<Vec<AndroidDensityDto>>("densities")
                .map(|vec| vec.into_iter().collect::<BTreeSet<_>>());
            let night = th.optional("night");
            let legacy_loader = th.optional::<bool>("legacy_loader");
            th.finalize(None)?;
            // endregion: extract

            // region: validate
            let remote_id = validate_remote_id(remote_id, ctx.declared_remote_ids)?;
            // endregion: validate

            Ok(Self {
                remote_id,
                android_res_dir,
                quality,
                densities,
                night,
                legacy_loader,
            })
        }
    }

    impl<'de> Deserialize<'de> for AndroidDensityDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            match value.as_str() {
                Some("ldpi") => Ok(AndroidDensityDto::LDPI),
                Some("mdpi") => Ok(AndroidDensityDto::MDPI),
                Some("hdpi") => Ok(AndroidDensityDto::HDPI),
                Some("xhdpi") => Ok(AndroidDensityDto::XHDPI),
                Some("xxhdpi") => Ok(AndroidDensityDto::XXHDPI),
                Some("xxxhdpi") => Ok(AndroidDensityDto::XXXHDPI),
                _ => Err(expected("android density name: `*dpi`", value.take(), value.span).into()),
            }
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
    fn AndroidWebpProfileDto__valid_fully_defined_toml__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        remote = "figma"
        android_res_dir = "src/main/res"
        quality = 100
        densities = ["ldpi", "mdpi", "hdpi", "xhdpi", "xxhdpi", "xxxhdpi"]
        night = "{base} / dark"
        legacy_loader = false
        "#;
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let expected_dto = AndroidWebpProfileDto {
            remote_id: Some("figma".to_string()),
            android_res_dir: Some(PathBuf::from("src/main/res")),
            quality: Some(WebpQuality(100.0)),
            densities: {
                use AndroidDensityDto::*;
                Some(
                    [LDPI, MDPI, HDPI, XHDPI, XXHDPI, XXXHDPI]
                        .into_iter()
                        .collect(),
                )
            },
            night: Some(SingleNamePattern("{base} / dark".to_string())),
            legacy_loader: Some(true),
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let ctx = AndroidWebpProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_dto = AndroidWebpProfileDto::parse_with_ctx(&mut value, ctx).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn AndroidWebpProfileDto__valid_empty_toml__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        "#;
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let expected_dto = AndroidWebpProfileDto {
            remote_id: None,
            android_res_dir: None,
            quality: None,
            densities: None,
            night: None,
            legacy_loader: None,
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let ctx = AndroidWebpProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_dto = AndroidWebpProfileDto::parse_with_ctx(&mut value, ctx).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn AndroidWebpProfileDto__valid_invalid_remote__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                remote = "undeclared"
                quality = 75
            "#,
        );
        let declared_remote_ids: HashSet<_> = ["figma".to_string()].into_iter().collect();
        let err_spans = [Span::new(10, 20)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let ctx = AndroidWebpProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_err = AndroidWebpProfileDto::parse_with_ctx(&mut value, ctx).unwrap_err();

        // Then
        assert_eq!(err_spans.len(), actual_err.errors.len());
        for (expected_span, actual_err) in err_spans.into_iter().zip(actual_err.errors) {
            assert_eq!(expected_span, actual_err.span);
        }
    }

    #[test]
    fn AndroidWebpProfileDto__valid_undeclared_key__EXPECT__error_with_correct_span() {
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
        let ctx = AndroidWebpProfileDtoContext {
            declared_remote_ids: &declared_remote_ids,
        };
        let actual_err = AndroidWebpProfileDto::parse_with_ctx(&mut value, ctx).unwrap_err();

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
