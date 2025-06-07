#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct VariantsDto {
    pub naming: Option<VariantNamingDto>,
    pub list: Option<Vec<String>>,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct VariantNamingDto {
    pub local_name: String,
    pub figma_name: String,
}

pub(super) mod de {
    use super::*;
    use crate::parser::util::validate_non_empty;
    use toml_span::{Deserialize, ErrorKind, de_helpers::TableHelper};

    pub fn parse_variants(
        th: &mut TableHelper,
    ) -> std::result::Result<Option<VariantsDto>, toml_span::DeserError> {
        // region: extract
        let naming = th.optional::<VariantNamingDto>("variant_naming");
        let variants = th.optional_s::<Vec<String>>("variants");
        // endregion: extract

        // region: validate
        match (naming, variants) {
            (None, None) => Ok(None),
            (naming, None) => Ok(Some(VariantsDto { naming, list: None })),
            (naming, Some(list)) => {
                let list = validate_non_empty(Some(list), || {
                    "`variants` list cannot be empty".to_string()
                })?;
                Ok(Some(VariantsDto { naming, list }))
            }
        }
        // endregion: validate
    }

    impl<'de> Deserialize<'de> for VariantNamingDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let local_name = th.required_s::<String>("local_name")?;
            let figma_name = th.required_s::<String>("figma_name")?;
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
                n if !n.value.contains("{variant}") => {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom("expected string pattern with `{variant}` marker".into()),
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
                n if !n.value.contains("{variant}") => {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom("expected string pattern with `{variant}` marker".into()),
                        n.span,
                    ))
                    .into());
                }
                n => n.value,
            };
            // endregion: validate

            Ok(Self {
                local_name,
                figma_name,
            })
        }
    }
}
