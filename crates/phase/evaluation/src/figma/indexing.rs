use crate::{
    Error, Result,
    figma::{NodeMetadata, RemoteMetadata},
};
use dashmap::DashMap;
use lib_cache::{Cache, CacheKey};
use lib_figma_fluent::{FigmaApi, GetFileNodesQueryParameters};
use log::debug;
use phase_loading::RemoteSource;
use std::{collections::HashMap, sync::Arc};

pub struct RemoteIndex {
    api: FigmaApi,
    cache: Cache,
    index: Arc<DashMap<String, NodeMetadata>>,
}

pub enum Subscription<'a> {
    FromCache(HashMap<String, NodeMetadata>),
    FromRemote(Box<dyn Iterator<Item = Result<NodeMetadata>> + Send + 'a>),
}

#[must_use]
pub struct SubscriptionHandle(CacheKey, Arc<DashMap<String, NodeMetadata>>, Cache);

impl RemoteIndex {
    pub const REMOTE_SOURCE_TAG: u8 = 0x42;

    pub fn new(api: FigmaApi, cache: Cache) -> Self {
        Self {
            api,
            cache,
            index: Arc::new(DashMap::with_capacity(1024)),
        }
    }

    /// This function  must be called from one thread per remote only
    pub fn subscribe<'a>(
        &'a self,
        remote: &'a RemoteSource,
        refetch: bool,
    ) -> Result<(SubscriptionHandle, Subscription<'a>)> {
        // construct unique cache key
        let cache_key = CacheKey::builder()
            .set_tag(Self::REMOTE_SOURCE_TAG)
            .write_str(&remote.file_key)
            .write_str(&remote.container_node_ids.join(","))
            .build();

        // return cached value if it exists
        if !refetch {
            if let Some(metadata) = self.cache.get::<RemoteMetadata>(&cache_key)? {
                return Ok((
                    SubscriptionHandle(cache_key, self.index.clone(), self.cache.clone()),
                    Subscription::FromCache(metadata.name_to_node),
                ));
            }
        }

        debug!(target: "Updating", "remote index {remote}");
        let stream = self.api.get_file_nodes(
            &remote.access_token,
            &remote.file_key,
            GetFileNodesQueryParameters {
                ids: Some(&remote.container_node_ids),
                geometry: Some("paths"),
                ..Default::default()
            },
        )?;

        let iter = stream.filter_map(|item| match item {
            Ok(node) => {
                // Ignore nodes which are not components or are not visible, do not store them in the index
                if node.r#type != "COMPONENT" || !node.visible {
                    return None;
                }
                let node = NodeMetadata {
                    id: node.id,
                    name: node.name,
                    hash: node.hash,
                    uses_raster_paints: node.has_raster_fills,
                };
                if !self.index.contains_key(&node.name) {
                    self.index.insert(node.name.to_owned(), node.clone());
                    Some(Ok(node))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(Error::IndexingRemote(e.to_string()))),
        });

        Ok((
            SubscriptionHandle(cache_key, self.index.clone(), self.cache.clone()),
            Subscription::FromRemote(Box::new(iter)),
        ))
    }
}

impl SubscriptionHandle {
    pub fn commit_cache(self) -> Result<()> {
        let SubscriptionHandle(cache_key, index, cache) = self;

        let metadata = RemoteMetadata {
            name_to_node: index
                .iter()
                .map(|it| (it.key().to_owned(), it.value().to_owned()))
                .collect(),
        };
        // remember result to cache
        cache.put::<RemoteMetadata>(&cache_key, &metadata)?;
        Ok(())
    }
}
