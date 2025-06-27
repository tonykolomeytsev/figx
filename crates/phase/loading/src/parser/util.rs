use std::collections::HashSet;
use toml_span::{ErrorKind, Spanned};

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

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use toml_span::Span;

    use super::*;

    #[test]
    fn validate_non_empty__valid_data__ok() {
        // Given
        let valid_value = Some(Spanned {
            value: vec![1, 2, 3],
            span: Span::new(0, 1),
        });
        
        // When
        let result = validate_non_empty(valid_value, || "".to_string());
        
        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn validate_non_empty__INvalid_data__err() {
        // Given
        let valid_value: Option<Spanned<Vec<i32>>> = Some(Spanned {
            value: vec![],
            span: Span::new(0, 1),
        });
        
        // When
        let result = validate_non_empty(valid_value, || "ERROR".to_string());
        
        // Then
        assert!(result.is_err());
    }
}