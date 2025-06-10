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
