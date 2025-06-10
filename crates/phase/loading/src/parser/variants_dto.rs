use std::collections::BTreeMap;
use crate::CanBeExtendedBy;

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct VariantsDto {
    pub all_variants: Option<BTreeMap<String, VariantDto>>,
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
    pub output_name: String,
    pub figma_name: String,
    pub scale: Option<f32>,
}

pub(super) mod de {
    use super::*;
    use crate::parser::util::{validate_figma_scale, validate_non_empty};
    use toml_span::{Deserialize, ErrorKind, de_helpers::TableHelper};

    impl<'de> Deserialize<'de> for VariantsDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let use_variants = th.optional_s::<Vec<String>>("use");
            let mut variants = th.table;
            // endregion: extract

            // region: validate
            let mut all_variants = BTreeMap::new();
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
            let r#use =
                validate_non_empty(use_variants, || "variants list cannot be empty".to_string())?;
            // endregion: validate

            Ok(Self {
                all_variants,
                use_variants: r#use,
            })
        }
    }

    impl<'de> Deserialize<'de> for VariantDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let local_name = th.required_s::<String>("output_name")?;
            let figma_name = th.required_s::<String>("figma_name")?;
            let scale = th.optional_s::<f32>("scale");
            th.finalize(None)?;
            // endregion: extract

            // region: validate
            let local_name = match local_name {
                n if !n.value.contains("{base}") => {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom("expected string pattern with `{base}` marker".into()),
                        n.span,
                    ))
                    .into());
                }
                n => n.value,
            };
            let figma_name = match figma_name {
                n if !n.value.contains("{base}") => {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom("expected string pattern with `{base}` marker".into()),
                        n.span,
                    ))
                    .into());
                }
                n => n.value,
            };
            let scale = validate_figma_scale(scale)?;
            // endregion: validate

            Ok(Self {
                output_name: local_name,
                figma_name,
                scale,
            })
        }
    }
}
