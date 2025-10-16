use crate::parser::{AccessTokenDefinitionDto, NodeIdListDto};
use ordermap::OrderMap;
use toml_span::Span;

#[derive(Default)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct RemotesDto(pub OrderMap<String, RemoteDto>);

#[derive(Default)]
pub struct RemotesDtoContext {
    pub ignore_missing_access_token: bool,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct RemoteDto {
    pub file_key: String,
    pub container_node_ids: NodeIdListDto,
    pub access_token: AccessTokenDefinitionDto,
    pub default: Option<bool>,
    pub key_span: Span,
}

mod de {
    use super::*;
    use crate::ParseWithContext;
    use ordermap::OrderMap;
    use toml_span::{Deserialize, ErrorKind, de_helpers::TableHelper};

    impl<'de> ParseWithContext<'de> for RemotesDto {
        type Context = RemotesDtoContext;

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let mut remotes = OrderMap::with_capacity(th.table.len()); // ordermap for deterministic order
            for (key, value) in th.table.iter_mut() {
                let remote_id = key.to_string();
                let mut remote = RemoteDto::parse_with_ctx(value, ())?;
                remote.key_span = key.span;
                remotes.insert(remote_id, (key.clone(), remote));
            }
            th.finalize(Some(value))?;
            // endregion: extract

            // region: validate
            if remotes.len() > 1 {
                let default_remotes = remotes
                    .iter()
                    .filter(|(_, v)| v.1.default == Some(true))
                    .map(|(_, v)| v)
                    .collect::<Vec<_>>();
                if default_remotes.len() == 0 {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom("at least one remote should be marked as default".into()),
                        // Span of the first remote
                        remotes.iter().next().unwrap().1.0.span.clone(),
                    ))
                    .into());
                }
                if default_remotes.len() > 1 {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom("only one remote can be marked as default".into()),
                        // Span of the last remote marked as default
                        default_remotes.iter().last().unwrap().0.span.clone(),
                    ))
                    .into());
                }
            }
            // endregion: validate

            Ok(Self(
                remotes
                    .into_iter()
                    .map(|(k, (_, mut remote))| {
                        if ctx.ignore_missing_access_token {
                            remote.access_token =
                                AccessTokenDefinitionDto::Explicit(":)".to_owned());
                        }
                        (k, remote)
                    })
                    .collect(),
            ))
        }
    }

    impl<'de> ParseWithContext<'de> for RemoteDto {
        type Context = ();

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            _ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let file_key = th.required_s::<String>("file_key")?;
            let container_node_ids = th.required_s::<NodeIdListDto>("container_node_ids")?.value;
            let access_token = if let Some((_, mut value)) = th.take("access_token") {
                AccessTokenDefinitionDto::deserialize(&mut value)?
            } else {
                AccessTokenDefinitionDto::default()
            };
            let default = th.optional("default");
            th.finalize(None)?;
            // endregion: extract

            // region: validate
            let file_key = match file_key.value.as_str() {
                "" => {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom("file_key cannot be empty".into()),
                        file_key.span,
                    ))
                    .into());
                }
                s => s.to_owned(),
            };
            // endregion: validate

            Ok(Self {
                file_key,
                container_node_ids,
                access_token,
                default,
                key_span: Default::default(),
            })
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {

    use super::*;
    use crate::ParseWithContext;
    use toml_span::{ErrorKind, Span};
    use unindent::unindent;

    #[test]
    fn RemotesDto__parse_fully_defined_remotes__EXPECT__valid_dto() {
        // Given
        let toml = unindent(
            r#"
                [icons]
                file_key = "abcdefg"
                container_node_ids = ["42-42"]
                access_token = "fig_123456789"
                default = true

                [illustrations]
                file_key = "hijklmno"
                container_node_ids = ["0-1"]
                access_token = "fig_987654321"
            "#,
        );
        let expected_dto = {
            let mut remotes = OrderMap::new();
            remotes.insert(
                "icons".to_owned(),
                RemoteDto {
                    file_key: "abcdefg".to_string(),
                    container_node_ids: NodeIdListDto::Plain(vec!["42-42".to_string()]),
                    access_token: AccessTokenDefinitionDto::Explicit("fig_123456789".to_string()),
                    default: Some(true),
                    key_span: Span::new(1, 6),
                },
            );
            remotes.insert(
                "illustrations".to_owned(),
                RemoteDto {
                    file_key: "hijklmno".to_string(),
                    container_node_ids: NodeIdListDto::Plain(vec!["0-1".to_string()]),
                    access_token: AccessTokenDefinitionDto::Explicit("fig_987654321".to_string()),
                    default: None,
                    key_span: Span::new(108, 121),
                },
            );
            RemotesDto(remotes)
        };

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let actual_dto = RemotesDto::parse_with_ctx(&mut value, Default::default()).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn RemotesDto__parse_remotes_with_no_default__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                [icons]
                file_key = "abcdefg"
                container_node_ids = ["42-42"]
                access_token = "fig_123456789"
                # default = true ## NO DEFAULT :)

                [illustrations]
                file_key = "hijklmno"
                container_node_ids = ["0-1"]
                access_token = "fig_987654321"
            "#,
        );
        let expected_spans = [Span::new(1, 6)]; // Highlight first key

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let actual_err = RemotesDto::parse_with_ctx(&mut value, Default::default()).unwrap_err();

        // Then
        for (err, expected_span) in actual_err.errors.iter().zip(expected_spans) {
            assert_eq!(expected_span, err.span);
        }
    }

    #[test]
    fn RemotesDto__parse_remotes_with_too_much_default__EXPECT__error_with_correct_spans() {
        // Given
        let toml = unindent(
            r#"
                [icons]
                file_key = "abcdefg"
                container_node_ids = ["42-42"]
                access_token = "fig_123456789"
                default = true

                [illustrations]
                file_key = "hijklmno"
                container_node_ids = ["0-1"]
                access_token = "fig_987654321"
                default = true
            "#,
        );
        let expected_spans = [Span::new(108, 121)]; // Highlight last key

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let actual_err = RemotesDto::parse_with_ctx(&mut value, Default::default()).unwrap_err();

        // Then
        for (err, expected_span) in actual_err.errors.iter().zip(expected_spans) {
            assert_eq!(expected_span, err.span);
        }
    }

    #[test]
    fn RemoteDto__parse_valid_remote__EXPECT__valid_dto() {
        // Given
        let toml = r#"
        file_key = "abcdefg"
        container_node_ids = ["42-42"]
        access_token = "fig_123456789"
        default = true
        "#;
        let expected_dto = RemoteDto {
            file_key: "abcdefg".to_string(),
            container_node_ids: NodeIdListDto::Plain(vec!["42-42".to_string()]),
            access_token: AccessTokenDefinitionDto::Explicit("fig_123456789".to_string()),
            default: Some(true),
            key_span: Default::default(),
        };

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let actual_dto = RemoteDto::parse_with_ctx(&mut value, ()).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn RemoteDto__parse_remote_w_wrong_fields__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                file_key = 123456789
                container_node_ids = ["42-42"]
                access_token = "fig_123456789"
            "#,
        );
        let expected_spans = [Span::new(11, 20)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let actual_err = RemoteDto::parse_with_ctx(&mut value, ()).unwrap_err();

        // Then
        for (err, expected_span) in actual_err.errors.iter().zip(expected_spans) {
            assert_eq!(expected_span, err.span);
        }
    }

    #[test]
    fn RemoteDto__parse_remote_w_empty_file_key__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                file_key = ""
                container_node_ids = ["42-42"]
                access_token = "fig_123456789"
            "#,
        );
        let expected_spans = [Span::new(11, 12)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let actual_err = RemoteDto::parse_with_ctx(&mut value, ()).unwrap_err();

        // Then
        for (err, expected_span) in actual_err.errors.iter().zip(expected_spans) {
            assert_eq!(expected_span, err.span);
        }
    }

    #[test]
    fn RemoteDto__parse_remote_w_empty_node_ids__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                file_key = "abcdefg"
                container_node_ids = []
                access_token = "fig_123456789"
            "#,
        );
        let expected_spans = [Span::new(42, 44)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let actual_err = RemoteDto::parse_with_ctx(&mut value, ()).unwrap_err();

        // Then
        for (err, expected_span) in actual_err.errors.iter().zip(expected_spans) {
            assert_eq!(expected_span, err.span);
        }
    }

    #[test]
    fn RemoteDto__parse_remote_w_empty_node_id__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                file_key = "abcdefg"
                container_node_ids = ["42-42", ""]
                access_token = "fig_123456789"
            "#,
        );
        let expected_spans = [Span::new(52, 53)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let actual_err = RemoteDto::parse_with_ctx(&mut value, ()).unwrap_err();

        // Then
        for (err, expected_span) in actual_err.errors.iter().zip(expected_spans) {
            assert_eq!(expected_span, err.span);
        }
    }

    #[test]
    fn RemoteDto__parse_remote_w_empty_access_token__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                file_key = "abcdefg"
                container_node_ids = ["42-42"]
                access_token = ""
            "#,
        );
        let expected_spans = [Span::new(67, 68)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let actual_err = RemoteDto::parse_with_ctx(&mut value, ()).unwrap_err();

        // Then
        for (err, expected_span) in actual_err.errors.iter().zip(expected_spans) {
            assert_eq!(expected_span, err.span);
        }
    }

    #[test]
    fn RemoteDto__undeclared_keys__EXPECT__error_with_correct_span() {
        // Given
        let toml = unindent(
            r#"
                file_key = "abcdefg"
                container_node_ids = ["42-42"]
                some_access_token_idk = "12345"
                smth_lmao = 1
            "#,
        );
        let expected_spans = [Span::new(84, 93), Span::new(52, 73)];

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let actual_err = RemoteDto::parse_with_ctx(&mut value, ()).unwrap_err();

        // Then
        for err in actual_err.errors {
            if let toml_span::Error {
                kind: ErrorKind::UnexpectedKeys { keys, .. },
                ..
            } = err
            {
                for ((_, actual_span), expected_span) in keys.iter().zip(expected_spans) {
                    assert_eq!(expected_span, *actual_span);
                }
            }
        }
    }
}
