use super::ToCacheKey;
use super::volatile_action;
use crate::Error;
use crate::EvalState;
use crate::Result;
use bincode::Decode;
use bincode::Encode;
use lib_cache::CacheKey;
use lib_figma::GetFileNodesQueryParameters;
use lib_figma::Node;
use lib_graph_exec::action::Action;
use lib_graph_exec::action::ActionDiagnostics;
use lib_graph_exec::action::ExecutionContext;
use lib_pretty::State::*;
use log::debug;
use phase_loading::RemoteSource;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

pub struct FetchRemoteAction {
    pub remote: Arc<RemoteSource>,
    pub force_refetch: bool,
}

impl Action<CacheKey, Error, EvalState> for FetchRemoteAction {
    fn execute(&self, ctx: ExecutionContext<CacheKey, EvalState>) -> Result<CacheKey> {
        volatile_action(
            &ctx.state.cache,
            self.remote.to_cache_key(),
            self.force_refetch,
            |cache_key| self.fetch_remote_impl(cache_key, &ctx.state),
        )
    }

    fn diagnostics_info(&self) -> ActionDiagnostics {
        ActionDiagnostics {
            name: "Fetch Figma file metadata".to_string(),
            params: vec![
                ("id".to_string(), self.remote.id.clone()),
                ("file_key".to_string(), self.remote.file_key.clone()),
                (
                    "nodes".to_string(),
                    format!("[{}]", self.remote.container_node_ids.join(", ")),
                ),
            ],
        }
    }
}

#[derive(Debug, Encode, Decode)]
pub struct RemoteMetadata {
    pub name_to_node: HashMap<String, NodeMetadata>,
}

#[derive(Debug, Encode, Decode)]
pub struct NodeMetadata {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub hash: u64,
}

impl FetchRemoteAction {
    pub const TAG: u8 = 0x00;

    fn fetch_remote_impl(&self, stable_cache_key: CacheKey, state: &EvalState) -> Result<CacheKey> {
        let FetchRemoteAction {
            remote,
            force_refetch: _,
        } = &self;
        debug!("Fetching remote with id '{}'...", remote.id);

        // If no cache, fetching remote from Figma API
        let ui_state = state.renderer.get_handle();
        ui_state.set_state(Fetching(format!("{remote}")));

        let response = state.figma_api.get_file_nodes(
            &remote.access_token,
            &remote.file_key,
            GetFileNodesQueryParameters {
                ids: Some(remote.container_node_ids.clone()),
                geometry: Some("paths".to_string()),
                ..Default::default()
            },
        )?;
        let all_nodes: Vec<Node> = response
            .nodes
            .into_values()
            .map(|node| node.document)
            .collect();
        let metadata = extract_metadata(&all_nodes)?;
        let mut volatile_cache_key = CacheKey::builder()
            .set_tag(Self::TAG)
            .write(stable_cache_key.as_ref());
        for node in all_nodes {
            volatile_cache_key = volatile_cache_key.write_u64(node.hash);
        }
        let volatile_cache_key = volatile_cache_key.build();
        state.cache.put(&volatile_cache_key, &metadata)?;
        debug!("Remote {remote} fetched successfully");
        Ok(volatile_cache_key)
    }
}

fn extract_metadata(values: &[Node]) -> Result<RemoteMetadata> {
    let mut queue = VecDeque::new();
    let mut name_to_node = HashMap::new();
    for value in values {
        queue.push_back(value);
    }
    while let Some(current) = queue.pop_front() {
        if !current.name.is_empty() {
            name_to_node.insert(
                current.name.clone(),
                NodeMetadata {
                    id: current.id.clone(),
                    name: current.name.clone(),
                    visible: current.visible,
                    hash: current.hash,
                },
            );
        }
        for child in &current.children {
            queue.push_back(child);
        }
    }
    Ok(RemoteMetadata { name_to_node })
}
