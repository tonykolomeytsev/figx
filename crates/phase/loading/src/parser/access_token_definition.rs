use toml_span::{Deserialize, ErrorKind, Value, de_helpers::TableHelper, value::ValueInner};

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) enum AccessTokenDefinitionDto {
    Explicit(String),
    Env(String),
    Keychain,
    Priority(Vec<AccessTokenDefinitionDto>),
}

impl Default for AccessTokenDefinitionDto {
    fn default() -> Self {
        Self::Env("FIGMA_PERSONAL_TOKEN".to_owned())
    }
}

impl<'de> Deserialize<'de> for AccessTokenDefinitionDto {
    fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
        let span = value.span;
        match value.take() {
            ValueInner::Array(arr) => {
                let priority = arr
                    .into_iter()
                    .map(|mut it| AccessTokenDefinitionDto::deserialize(&mut it))
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(Self::Priority(priority));
            }
            ValueInner::String(s) => {
                if s.is_empty() {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom("access token cannot be empty".into()),
                        value.span,
                    ))
                    .into());
                }
                return Ok(Self::Explicit(s.to_string()));
            }
            v => {
                let mut value = Value::with_span(v, span);
                let mut th = TableHelper::new(&mut value)?;
                if th.contains("env") {
                    let env = th.required_s::<String>("env")?;
                    if env.value.is_empty() {
                        return Err(toml_span::Error::from((
                            ErrorKind::Custom(
                                "access token environment variable name cannot be empty".into(),
                            ),
                            env.span,
                        ))
                        .into());
                    }
                    return Ok(Self::Env(env.value));
                } else if th.contains("keychain") {
                    let keychain = th.required_s::<bool>("keychain")?;
                    if !keychain.value {
                        return Err(toml_span::Error::from((
                            ErrorKind::Custom("only `keychain = true` syntax supported".into()),
                            keychain.span,
                        ))
                        .into());
                    }
                    return Ok(Self::Keychain);
                } else {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom(
                            "expected `{ env = \"SOME_ENV\" }` or `{ keychain = true }`".into(),
                        ),
                        value.span,
                    ))
                    .into());
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {

    use super::*;
    use unindent::unindent;

    #[test]
    fn AccessTokenDefinitionDto__explicit_str__EXPECT__ok() {
        // Given
        let toml = unindent(
            r#"
                access_token = "fig_987654321"
            "#,
        );
        let expected_dto = AccessTokenDefinitionDto::Explicit("fig_987654321".to_string());

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let mut value = value.pointer_mut("/access_token").unwrap();
        let actual_dto = AccessTokenDefinitionDto::deserialize(&mut value).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn AccessTokenDefinitionDto__env_variable__EXPECT__ok() {
        // Given
        let toml = unindent(
            r#"
                access_token.env = "ENV_ENV_ENV"
            "#,
        );
        let expected_dto = AccessTokenDefinitionDto::Env("ENV_ENV_ENV".to_string());

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let mut value = value.pointer_mut("/access_token").unwrap();
        let actual_dto = AccessTokenDefinitionDto::deserialize(&mut value).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn AccessTokenDefinitionDto__keychain_enabled__EXPECT__ok() {
        // Given
        let toml = unindent(
            r#"
                access_token.keychain = true
            "#,
        );
        let expected_dto = AccessTokenDefinitionDto::Keychain;

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let mut value = value.pointer_mut("/access_token").unwrap();
        let actual_dto = AccessTokenDefinitionDto::deserialize(&mut value).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn AccessTokenDefinitionDto__priority__EXPECT__ok() {
        // Given
        let toml = unindent(
            r#"
                access_token = [ 
                    { env = "ENV_ENV_ENV" }, 
                    { keychain = true },
                    "fallback_key",
                ]
            "#,
        );
        let expected_dto = AccessTokenDefinitionDto::Priority(vec![
            AccessTokenDefinitionDto::Env("ENV_ENV_ENV".to_string()),
            AccessTokenDefinitionDto::Keychain,
            AccessTokenDefinitionDto::Explicit("fallback_key".to_string()),
        ]);

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let mut value = value.pointer_mut("/access_token").unwrap();
        let actual_dto = AccessTokenDefinitionDto::deserialize(&mut value).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }
}
