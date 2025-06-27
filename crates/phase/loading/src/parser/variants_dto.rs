use crate::{CanBeExtendedBy, ExportScale, SingleNamePattern};
use ordermap::OrderMap;

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct VariantsDto {
    pub all_variants: Option<OrderMap<String, VariantDto>>,
    pub use_variants: Option<Vec<String>>,
}

impl CanBeExtendedBy<VariantsDto> for VariantsDto {
    fn extend(&self, another: &VariantsDto) -> Self {
        Self {
            all_variants: another
                .all_variants
                .as_ref()
                .or(self.all_variants.as_ref())
                .cloned(),
            use_variants: another
                .use_variants
                .as_ref()
                .or(self.use_variants.as_ref())
                .cloned(),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct VariantDto {
    pub output_name: SingleNamePattern,
    pub figma_name: SingleNamePattern,
    pub scale: Option<ExportScale>,
}

#[cfg(test)]
#[macro_export]
macro_rules! variant_dto {
    ($out:literal <- $fig:literal) => {
        crate::parser::VariantDto {
            output_name: crate::SingleNamePattern($out.to_owned()),
            figma_name: crate::SingleNamePattern($fig.to_owned()),
            scale: None,
        }
    };
    ($out:literal <- $fig:literal (x$scale:literal)) => {
        crate::parser::VariantDto {
            output_name: crate::SingleNamePattern($out.to_owned()),
            figma_name: crate::SingleNamePattern($fig.to_owned()),
            scale: Some(crate::ExportScale($scale)),
        }
    };
}

pub(super) mod de {
    use super::*;
    use crate::{ExportScale, parser::util::validate_non_empty};
    use toml_span::{Deserialize, de_helpers::TableHelper};

    impl<'de> Deserialize<'de> for VariantsDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let use_variants = th.optional_s::<Vec<String>>("use");
            let mut variants = th.table;
            // endregion: extract

            // region: validate
            let mut all_variants = OrderMap::new();
            for (k, v) in variants.iter_mut() {
                let variant_key = k.name.to_string();
                let variant_value = VariantDto::deserialize(v)?;
                all_variants.insert(variant_key, variant_value);
            }
            let all_variants = if all_variants.is_empty() {
                None
            } else {
                Some(all_variants)
            };
            let use_variants =
                validate_non_empty(use_variants, || "variants list cannot be empty".to_string())?;
            // endregion: validate

            Ok(Self {
                all_variants,
                use_variants,
            })
        }
    }

    impl<'de> Deserialize<'de> for VariantDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let output_name = th.required::<SingleNamePattern>("output_name")?;
            let figma_name = th.required::<SingleNamePattern>("figma_name")?;
            let scale = th.optional::<ExportScale>("scale");
            th.finalize(None)?;
            // endregion: extract

            Ok(Self {
                output_name,
                figma_name,
                scale,
            })
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use super::*;
    use ordermap::ordermap;
    use toml_span::de_helpers::TableHelper;

    #[test]
    fn VariantsDto__empty_variants__EXPECT__none() {
        // Given
        let toml = "[variants]";

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let variants = TableHelper::new(&mut value)
            .unwrap()
            .optional::<VariantsDto>("variants");

        // Then
        assert_eq!(
            Some(VariantsDto {
                all_variants: None,
                use_variants: None
            }),
            variants
        );
    }

    #[test]
    fn VariantsDto__valid_variants__EXPECT__predictable_result() {
        // Given
        let toml = r#"
        [variants]
        use = ["x1", "x2", "x3"]
        x1 = { output_name = "{base}", figma_name = "{base}", scale = 1.0 }
        x2 = { output_name = "{base}", figma_name = "{base}", scale = 2.0 }
        x3 = { output_name = "{base}", figma_name = "{base}", scale = 3.0 }
        "#;

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let variants = TableHelper::new(&mut value)
            .unwrap()
            .required::<VariantsDto>("variants")
            .unwrap();

        // Then
        assert_eq!(
            Some(vec!["x1".to_string(), "x2".to_string(), "x3".to_string()]),
            variants.use_variants
        );
        assert_eq!(
            Some(ordermap! {
                "x1".to_string() => VariantDto { output_name: "{base}".into(), figma_name: "{base}".into(), scale: Some(ExportScale(1.0)) },
                "x2".to_string() => VariantDto { output_name: "{base}".into(), figma_name: "{base}".into(), scale: Some(ExportScale(2.0)) },
                "x3".to_string() => VariantDto { output_name: "{base}".into(), figma_name: "{base}".into(), scale: Some(ExportScale(3.0)) },
            }),
            variants.all_variants
        );
    }

    #[test]
    fn VariantsDto__one_variant_extend_another__EXPECT__predictable_result() {
        // Given
        let first = VariantsDto {
            all_variants: None,
            use_variants: Some(vec!["x1".to_string(), "x2".to_string()]),
        };
        let second = VariantsDto {
            all_variants: Some(ordermap! {
                "x1".to_string() => VariantDto { output_name: "{base]1".into(), figma_name: "{base}_1".into(), scale: Some(ExportScale(1.0)) },
                "x2".to_string() => VariantDto { output_name: "{base]2".into(), figma_name: "{base}_2".into(), scale: Some(ExportScale(2.0)) },
            }),
            use_variants: None,
        };

        // When
        let third = first.extend(&second);

        // Then
        assert_eq!(
            VariantsDto {
                all_variants: Some(ordermap! {
                    "x1".to_string() => VariantDto { output_name: "{base]1".into(), figma_name: "{base}_1".into(), scale: Some(ExportScale(1.0)) },
                    "x2".to_string() => VariantDto { output_name: "{base]2".into(), figma_name: "{base}_2".into(), scale: Some(ExportScale(2.0)) },
                }),
                use_variants: Some(vec!["x1".to_string(), "x2".to_string()]),
            },
            third,
        );
    }

    impl From<&str> for SingleNamePattern {
        fn from(value: &str) -> Self {
            SingleNamePattern(value.to_owned())
        }
    }
}
