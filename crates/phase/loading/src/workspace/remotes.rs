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
