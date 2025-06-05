use crate::RemoteSource;
use crate::parser::{AccessTokenDefinitionDto, RemotesDto};
use crate::{Error, Result};
use ordermap::OrderMap;
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) fn parse_remotes(
    RemotesDto(remotes): RemotesDto,
) -> Result<OrderMap<String, Arc<RemoteSource>>> {
    let mut all_remotes: OrderMap<String, Arc<RemoteSource>> =
        OrderMap::with_capacity(remotes.capacity());

    for (id, dto) in &remotes {
        let remote = RemoteSource {
            id: id.clone(),
            file_key: dto.file_key.to_owned(),
            container_node_ids: dto.container_node_ids.to_owned(),
            access_token: match &dto.access_token {
                AccessTokenDefinitionDto::Explicit(token) => token.to_owned(),
                AccessTokenDefinitionDto::Env(env) => std::env::var(env).map_err(|_| {
                    Error::WorkspaceRemoteNoAccessToken(id.to_owned(), PathBuf::new())
                })?,
            },
        };
        all_remotes.insert(id.to_owned(), Arc::new(remote));
    }

    Ok(all_remotes)
}

#[cfg(test)]
#[allow(non_snake_case, unused)]
mod test {
    use ordermap::ordermap;

    use super::*;

    #[test]
    fn parse_valid_minimal_remote_definition__EXPECT__ok() {
        // Given
        let text = r#"
        [icons]
        file_key = "123456789"
        container_node_ids = ["1234-567890"]
        access_token = "hello"
        "#;
        let model = ordermap! {
            "icons".to_string() => Arc::new(RemoteSource {
                id: "icons".to_string(),
                file_key: "123456789".to_string(),
                container_node_ids: vec!["1234-567890".to_string()],
                access_token: "hello".to_string(),
            })
        };

        // When
        // let result = parse_remotes(toml::from_str(text).unwrap());

        // Then
        // assert_eq!(model, result.unwrap());
    }

    #[test]
    fn parse_valid_full_remote_definition__EXPECT__ok() {
        // Given
        let text = r#"
        [icons]
        file_key = "123456789"
        container_node_ids = ["1234-567890", "4242-181818"]
        access_token = "hello"

        [ui-kit]
        file_key = "abcdefghi"
        container_node_ids = ["0000-567890", "XXXX-181818"]
        access_token = "hello2"
        default = true
        "#;
        let model = ordermap! {
            "ui-kit".to_string() => Arc::new(RemoteSource {
                id: "ui-kit".to_string(),
                file_key: "abcdefghi".to_string(),
                container_node_ids: vec!["0000-567890".to_string(), "XXXX-181818".to_string()],
                access_token: "hello2".to_string(),
            }),
            "icons".to_string() => Arc::new(RemoteSource {
                id: "icons".to_string(),
                file_key: "123456789".to_string(),
                container_node_ids: vec!["1234-567890".to_string(), "4242-181818".to_string()],
                access_token: "hello".to_string(),
            }),
        };

        // When
        // let result = parse_remotes(toml::from_str(text).unwrap());

        // Then
        // assert_eq!(model, result.unwrap());
    }

    #[test]
    fn parse_invalid_minimal_remote_definition__EXPECT__err() {
        // Given
        let text = r#"
        [icons]
        file_key = "123456789"
        access_token = "hello"
        "#;

        // When
        // let result = parse_remotes(toml::from_str(text).unwrap());

        // Then
        // assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_remote_definition_with_empty_containers__EXPECT__err() {
        // Given
        let text = r#"
        [icons]
        file_key = "123456789"
        access_token = "hello"
        container_node_ids = []
        "#;

        // When
        // let result = parse_remotes(toml::from_str(text).unwrap());

        // Then
        // assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_full_remote_definition__EXPECT__err() {
        // Given
        let text = r#"
        [icons]
        file_key = "123456789"
        container_node_ids = ["1234-567890", "4242-181818"]
        access_token = "hello"

        [ui-kit]
        file_key = "abcdefghi"
        container_node_ids = ["0000-567890", "XXXX-181818"]
        access_token = "hello2"
        "#;

        // When
        // let result = parse_remotes(toml::from_str(text).unwrap());

        // Then
        // assert!(result.is_err());
    }
}
