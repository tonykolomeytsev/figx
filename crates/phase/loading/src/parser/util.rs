use std::collections::HashSet;
use toml_span::{ErrorKind, Spanned};

pub(crate) fn validate_figma_scale(
    scale: Option<Spanned<f32>>,
) -> Result<Option<f32>, toml_span::DeserError> {
    if let Some(scale) = scale {
        match scale.value {
            0.1..=4.0 => Ok(Some(scale.value)),
            _ => Err(toml_span::Error {
                kind: ErrorKind::Custom("scale mut be a number from 0.1 to 4".into()),
                span: scale.span,
                line_info: None,
            }
            .into()),
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn validate_webp_quality(
    quality: Option<Spanned<f32>>,
) -> Result<Option<f32>, toml_span::DeserError> {
    if let Some(quality) = quality {
        match quality.value {
            0.0..100.0 => Ok(Some(quality.value)),
            _ => Err(toml_span::Error {
                kind: ErrorKind::Custom("quality mut be a number from 0 to 100".into()),
                span: quality.span,
                line_info: None,
            }
            .into()),
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn validate_remote_id(
    remote_id: Option<Spanned<String>>,
    declared_remote_ids: &HashSet<String>,
) -> std::result::Result<Option<String>, toml_span::DeserError> {
    if let Some(remote_id) = &remote_id {
        if !declared_remote_ids.contains(&remote_id.value) {
            let expected = declared_remote_ids
                .iter()
                .map(|it| format!("'{it}'"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(toml_span::Error {
                kind: ErrorKind::Custom(
                    format!(
                        "undeclared remote '{}' used here, expected values: [{expected}]",
                        remote_id.value
                    )
                    .into(),
                ),
                span: remote_id.span,
                line_info: None,
            }
            .into());
        }
    }
    Ok(remote_id.map(|it| it.value))
}

pub(crate) fn validate_non_empty<T>(
    list: Option<Spanned<Vec<T>>>,
    msg: impl FnOnce() -> String,
) -> std::result::Result<Option<Vec<T>>, toml_span::DeserError> {
    if let Some(list) = &list {
        if list.value.is_empty() {
            return Err(
                toml_span::Error::from((ErrorKind::Custom(msg().into()), list.span)).into(),
            );
        }
    }
    Ok(list.map(|it| it.value))
}
