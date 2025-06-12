use std::collections::HashSet;
use toml_span::{ErrorKind, Spanned};

pub(crate) fn validate_remote_id(
    remote_id: Option<Spanned<String>>,
    declared_remote_ids: &HashSet<String>,
    restricted_remote_ids: Option<&HashSet<String>>,
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
        match restricted_remote_ids {
            Some(ids) if ids.contains(&remote_id.value) => {
                return Err(toml_span::Error {
                    kind: ErrorKind::Custom(
                        format!(
                            "remote `{}` marked as `raster_only`, you cannot use it for profile that work with vector graphics",
                            remote_id.value,
                        )
                        .into(),
                    ),
                    span: remote_id.span,
                    line_info: None,
                }
                .into());
            }
            _ => (),
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
