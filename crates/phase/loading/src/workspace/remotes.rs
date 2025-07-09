use crate::RemoteSource;
use crate::parser::{AccessTokenDefinitionDto, RemotesDto};
use crate::{Error, Result};
use lib_auth::get_token;
use log::debug;
use ordermap::OrderMap;
use std::path::PathBuf;
use std::sync::Arc;
use toml_span::Span;

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
            access_token: parse_access_token_definition(id, &dto.access_token, &dto.key_span)?,
        };
        all_remotes.insert(id.to_owned(), Arc::new(remote));
    }

    Ok(all_remotes)
}

fn parse_access_token_definition(
    id: &str,
    dto: &AccessTokenDefinitionDto,
    span: &Span,
) -> Result<String> {
    match &dto {
        AccessTokenDefinitionDto::Explicit(token) => {
            debug!(target: "Remotes", "use an explicitly specified token for remote `{id}`");
            Ok(token.to_owned())
        }
        AccessTokenDefinitionDto::Env(env) => {
            let result = std::env::var(env).map_err(|_| {
                Error::WorkspaceRemoteNoAccessToken(id.to_owned(), PathBuf::new(), *span)
            });
            if result.is_ok() {
                debug!(target: "Remotes", "take access token for remote `{id}` from env `{env}`");
            }
            result
        }
        AccessTokenDefinitionDto::Keychain => match get_token() {
            Ok(Some(token)) => {
                debug!(target: "Remotes", "take access token for remote `{id}` from platform keychain");
                Ok(token)
            }
            Ok(None) => Err(Error::WorkspaceRemoteEmptyKeychain(
                id.to_owned(),
                PathBuf::new(),
                *span,
            )),
            Err(e) => Err(Error::WorkspaceRemoteKeychainError(e)),
        },
        AccessTokenDefinitionDto::Priority(defs) => {
            for def in defs {
                if let Ok(token) = parse_access_token_definition(id, def, span) {
                    return Ok(token);
                }
            }
            Err(Error::WorkspaceRemoteNoAccessToken(
                id.to_owned(),
                PathBuf::new(),
                *span,
            ))
        }
    }
}
