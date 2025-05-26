use crate::RemoteSource;
use crate::{Error, Result};
use ordermap::OrderMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Deserialize, Default)]
pub(super) struct RemotesDto(HashMap<String, RemoteDto>);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoteDto {
    pub file_key: String,
    #[serde(default = "Default::default")]
    pub container_node_ids: Vec<String>,
    #[serde(default = "Default::default")]
    pub access_token: AccessTokenDefinition,
    pub default: Option<bool>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AccessTokenDefinition {
    Explicit(String),
    Env { env: String },
}

impl Default for AccessTokenDefinition {
    fn default() -> Self {
        Self::Env {
            env: "FIGMA_PERSONAL_TOKEN".to_owned(),
        }
    }
}

pub(crate) fn parse_remotes(
    RemotesDto(remotes): RemotesDto,
) -> Result<OrderMap<String, Arc<RemoteSource>>> {
    let mut has_default_remote = false;
    let mut all_remotes: OrderMap<String, Arc<RemoteSource>> =
        OrderMap::with_capacity(remotes.capacity());
    let remotes_len = remotes.len();

    for (id, dto) in remotes {
        let access_token = parse_access_token(&dto.access_token)
            .ok_or(Error::WorkspaceRemoteNoAccessToken(id.to_string()))?;
        if dto.container_node_ids.is_empty() {
            return Err(Error::WorkspaceRemoteWithEmptyNodeId);
        }
        let remote = RemoteSource {
            id: id.clone(),
            file_key: dto.file_key,
            container_node_ids: dto.container_node_ids,
            access_token,
        };

        match (remotes_len, has_default_remote, dto.default) {
            (1, _, _) => {
                has_default_remote = true;
                all_remotes.insert(remote.id.clone(), Arc::new(remote));
            }
            (_, false, Some(true)) => {
                has_default_remote = true;
                all_remotes.insert_before(0, remote.id.clone(), Arc::new(remote));
            }
            (_, true, Some(true)) => return Err(Error::WorkspaceMoreThanOneDefaultRemotes),
            (_, _, Some(false)) | (_, _, None) => {
                all_remotes.insert(remote.id.clone(), Arc::new(remote));
            }
        };
    }

    if !has_default_remote {
        return Err(Error::WorkspaceAtLeastOneDefaultRemote);
    }

    Ok(all_remotes)
}

fn parse_access_token(value: &AccessTokenDefinition) -> Option<String> {
    match value {
        AccessTokenDefinition::Explicit(token) => Some(token.to_owned()),
        AccessTokenDefinition::Env { env } => match std::env::var(env) {
            Ok(value) => Some(value),
            _ => None,
        },
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
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
        let result = parse_remotes(toml::from_str(text).unwrap());

        // Then
        assert_eq!(model, result.unwrap());
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
        let result = parse_remotes(toml::from_str(text).unwrap());

        // Then
        assert_eq!(model, result.unwrap());
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
        let result = parse_remotes(toml::from_str(text).unwrap());

        // Then
        assert!(result.is_err());
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
        let result = parse_remotes(toml::from_str(text).unwrap());

        // Then
        assert!(result.is_err());
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
        let result = parse_remotes(toml::from_str(text).unwrap());

        // Then
        assert!(result.is_err());
    }
}
