use std::collections::BTreeMap;

#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum NodeIdListDto {
    Plain(Vec<String>),
    IdToTag(BTreeMap<String, String>),
}

mod de {
    use std::collections::BTreeMap;

    use log::warn;
    use toml_span::{Deserialize, ErrorKind, Spanned, value::ValueInner};

    use crate::parser::NodeIdListDto;

    impl<'de> Deserialize<'de> for NodeIdListDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            // region: extract
            let span = value.span;
            let dto = match value.take() {
                ValueInner::Array(arr) => {
                    let plain_ids: Vec<Spanned<String>> = arr
                        .into_iter()
                        .map(|mut it| Spanned::<String>::deserialize(&mut it))
                        .collect::<Result<Vec<_>, _>>()?;
                    if plain_ids.is_empty() {
                        return Err(toml_span::Error::from((
                            ErrorKind::Custom("container_node_ids cannot be empty".into()),
                            span,
                        ))
                        .into());
                    }
                    NodeIdListDto::Plain(
                        plain_ids
                            .into_iter()
                            .map(|id| {
                                if id.value.is_empty() {
                                    Err(toml_span::Error::from((
                                        ErrorKind::Custom("node id cannot be empty".into()),
                                        id.span,
                                    )))
                                } else {
                                    Ok(id.value)
                                }
                            })
                            .collect::<std::result::Result<Vec<_>, _>>()?,
                    )
                }
                ValueInner::Table(table) => {
                    warn!(target: "Experimental", "tagged node ids is an experimental feature, api may change in the future");
                    let mut tagged_ids = BTreeMap::new();
                    for (k, v) in table.into_iter() {
                        if k.name.is_empty() {
                            return Err(toml_span::Error::from((
                                ErrorKind::Custom("node id cannot be empty".into()),
                                k.span,
                            ))
                            .into());
                        }
                        let mut v = v;
                        let v = Spanned::<String>::deserialize(&mut v)?;
                        if v.value.is_empty() {
                            return Err(toml_span::Error::from((
                                ErrorKind::Custom("tag for node id cannot be empty".into()),
                                v.span,
                            ))
                            .into());
                        }
                        tagged_ids.insert(k.name.to_string(), v.value);
                    }

                    if tagged_ids.is_empty() {
                        return Err(toml_span::Error::from((
                            ErrorKind::Custom("container_node_ids cannot be empty".into()),
                            span,
                        ))
                        .into());
                    }
                    NodeIdListDto::IdToTag(tagged_ids)
                }
                _ => {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom(
                            "container_node_ids must be a list of strings or a table (\"id\" = \"tag\")"
                                .into(),
                        ),
                        span,
                    ))
                    .into());
                }
            };
            // endregion: extract

            Ok(dto)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use super::*;
    use toml_span::Deserialize;
    use unindent::unindent;

    #[test]
    fn NodeIdListDto__plain_ids__EXPECT__ok() {
        // Given
        let toml = unindent(
            r#"
                container_node_ids = ["1", "2"]
            "#,
        );
        let expected_dto = NodeIdListDto::Plain(vec!["1".to_string(), "2".to_string()]);

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let mut value = value.pointer_mut("/container_node_ids").unwrap();
        let actual_dto = NodeIdListDto::deserialize(&mut value).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }

    #[test]
    fn NodeIdListDto__tagged_ids__EXPECT__ok() {
        // Given
        let toml = unindent(
            r#"
                container_node_ids."1:123" = "actions"
                container_node_ids."2:123" = "environment"
            "#,
        );
        let expected_dto = NodeIdListDto::IdToTag({
            let mut m = BTreeMap::new();
            m.insert("1:123".to_string(), "actions".to_string());
            m.insert("2:123".to_string(), "environment".to_string());
            m
        });

        // When
        let mut value = toml_span::parse(&toml).unwrap();
        let mut value = value.pointer_mut("/container_node_ids").unwrap();
        let actual_dto = NodeIdListDto::deserialize(&mut value).unwrap();

        // Then
        assert_eq!(expected_dto, actual_dto);
    }
}
