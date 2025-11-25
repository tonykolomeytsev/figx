use crate::{
    Result,
    figma::{NodeMetadata, RemoteMetadata},
};
use lib_cache::{Cache, CacheKey};
use lib_figma_fluent::{FigmaApi, GetFileNodesQueryParameters, ScannedNodeDto};
use log::debug;
use phase_loading::RemoteSource;
use std::collections::{HashMap, VecDeque};

pub struct RemoteIndex {
    api: FigmaApi,
    cache: Cache,
}

impl RemoteIndex {
    pub const REMOTE_SOURCE_TAG: u8 = 0x42;

    pub fn new(api: FigmaApi, cache: Cache) -> Self {
        Self { api, cache }
    }

    /// This function  must be called from one thread per remote only
    pub fn load<'a>(
        &'a self,
        remote: &'a RemoteSource,
        refetch: bool,
    ) -> Result<HashMap<String, NodeMetadata>> {
        let container_node_ids = remote.container_node_ids.to_string_id_list();
        // construct unique cache key
        let cache_key = CacheKey::builder()
            .set_tag(Self::REMOTE_SOURCE_TAG)
            .write_str(&remote.file_key)
            .write_str(&container_node_ids.join(","))
            .build();

        // return cached value if it exists
        if !refetch {
            if let Some(metadata) = self.cache.get::<RemoteMetadata>(&cache_key)? {
                return Ok(metadata.name_to_node);
            }
        }

        debug!(target: "Updating", "remote index {remote}");
        let response = self.api.get_file_nodes(
            &remote.access_token,
            &remote.file_key,
            GetFileNodesQueryParameters {
                ids: Some(&container_node_ids),
                geometry: Some("paths"),
                ..Default::default()
            },
        )?;
        let mut name_to_node = HashMap::with_capacity(4096);
        for (_, root) in response.nodes {
            let nodes = extract_metadata(&[root.document]);
            for node in nodes {
                name_to_node.insert(node.name.to_owned(), node);
            }
        }
        let metadata = RemoteMetadata { name_to_node };

        self.cache.put::<RemoteMetadata>(&cache_key, &metadata)?;
        Ok(metadata.name_to_node)
    }
}

/// Mapper from response to metadata
fn extract_metadata(values: &[ScannedNodeDto]) -> Vec<NodeMetadata> {
    let mut queue = VecDeque::new();
    let mut output_nodes = Vec::with_capacity(4096);
    for value in values {
        if value.visible {
            queue.push_back(value);
        }
    }
    while let Some(current) = queue.pop_front() {
        if !current.name.is_empty() && current.r#type == "COMPONENT" {
            if current.visible {
                output_nodes.push(NodeMetadata {
                    id: current.id.clone(),
                    name: current.name.clone(),
                    hash: current.hash,
                    uses_raster_paints: current.fills.iter().any(|it| it.r#type == "IMAGE"),
                });
            }
        }
        for child in &current.children {
            if child.visible {
                queue.push_back(child);
            }
        }
    }
    output_nodes
}
