use super::{NodeMetadata, RemoteMetadata};
use crate::{Error, Result};
use dashmap::DashMap;
use lib_cache::{Cache, CacheKey};
use lib_figma::{FigmaApi, GetFileNodesQueryParameters, GetImageQueryParameters, Node};
use phase_loading::RemoteSource;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct FigmaRepository {
    api: FigmaApi,
    cache: Cache,
    locks: Arc<DashMap<String, Mutex<()>>>,
    io_pool: Arc<ThreadPool>,
}

pub type DownloadUrl = String;

impl FigmaRepository {
    const REMOTE_SOURCE_TAG: u8 = 0x42;
    const EXPORTED_IMAGE_TAG: u8 = 0x43;
    const DOWNLOADED_IMAGE_TAG: u8 = 0x44;

    pub fn new(api: FigmaApi, cache: Cache) -> Self {
        Self {
            api,
            cache,
            locks: Default::default(),
            io_pool: Arc::new(ThreadPoolBuilder::new().build().unwrap()),
        }
    }

    pub fn get_remote(
        &self,
        remote: &RemoteSource,
        refetch: bool,
        on_fetch_start: impl FnOnce(),
    ) -> Result<RemoteMetadata> {
        // construct unique cache key
        let cache_key = CacheKey::builder()
            .set_tag(Self::REMOTE_SOURCE_TAG)
            .write_str(&remote.file_key)
            .write_str(&remote.container_node_ids.join(","))
            .build();

        // this section will be accessed by only one thread for one remote
        // could be abused by user creating billions of remotes :)
        let mutex = self.locks.entry(remote.id.clone()).or_default();
        let _guard = mutex.lock().unwrap();

        // return cached value if it exists
        if !refetch {
            if let Some(metadata) = self.cache.get::<RemoteMetadata>(&cache_key)? {
                return Ok(metadata);
            }
        }

        // otherwise, request value from remote
        on_fetch_start();
        let response = self.io_pool.install(|| {
            self.api.get_file_nodes(
                &remote.access_token,
                &remote.file_key,
                GetFileNodesQueryParameters {
                    ids: Some(&remote.container_node_ids),
                    geometry: Some("paths"),
                    ..Default::default()
                },
            )
        });

        let metadata = {
            let response = response?;
            let all_nodes: Vec<Node> = response
                .nodes
                .into_values()
                .map(|node| node.document)
                .collect();
            extract_metadata(&all_nodes)
        };

        // remember result to cache
        self.cache.put::<RemoteMetadata>(&cache_key, &metadata)?;
        // return result and release lock
        Ok(metadata)
    }

    pub fn export(
        &self,
        remote: &RemoteSource,
        node: &NodeMetadata,
        format: &str,
        scale: f32,
        on_export_start: impl FnOnce(),
    ) -> Result<DownloadUrl> {
        // construct unique cache key
        let cache_key = CacheKey::builder()
            .set_tag(Self::EXPORTED_IMAGE_TAG)
            .write_str(&remote.file_key)
            .write_str(&remote.container_node_ids.join(","))
            .write_str(&node.id)
            .write_u64(node.hash)
            .write_str(format)
            .write_str(&scale.to_string())
            .build();

        // this section will be accessed by only one thread for one node
        // could be abused by user creating billions of remotes :)
        let mutex = self.locks.entry(node.id.clone()).or_default();
        let _guard = mutex.lock().unwrap();

        // return cached value if it exists
        if let Some(url) = self.cache.get::<DownloadUrl>(&cache_key)? {
            return Ok(url);
        }

        // otherwise, request value from remote
        on_export_start();
        let response = self.io_pool.install(|| {
            self.api.get_image(
                &remote.access_token,
                &remote.file_key,
                GetImageQueryParameters {
                    ids: Some(&vec![node.id.to_owned()]),
                    scale: Some(scale),
                    format: Some(format),
                    ..Default::default()
                },
            )
        });

        let node_id = node.id.as_str();
        let url = {
            let mut response = response?;
            if let Some(error) = response.err {
                return Err(Error::ExportImage(format!(
                    "got response with error: {error}"
                )));
            }
            let download_url = match response.images.remove(node_id) {
                Some(url) => url,
                None => {
                    return Err(Error::ExportImage(format!(
                        "response has no requested node with id '{node_id}'"
                    )));
                }
            };
            match download_url {
                Some(url) => url,
                None => {
                    return Err(Error::ExportImage(format!(
                        "requested node with id '{node_id}' was not rendered by Figma backend",
                    )));
                }
            }
        };

        // remember result to cache
        self.cache.put::<DownloadUrl>(&cache_key, &url)?;
        // return result and release lock
        Ok(url)
    }

    pub fn download(
        &self,
        remote: &RemoteSource,
        url: &str,
    ) -> Result<Vec<u8>> {
        // construct unique cache key
        let cache_key = CacheKey::builder()
            .set_tag(Self::DOWNLOADED_IMAGE_TAG)
            .write_str(&url)
            .build();

        // this section will be accessed by only one thread for one node
        // could be abused by user creating billions of remotes :)
        let mutex = self.locks.entry(url.to_owned()).or_default();
        let _guard = mutex.lock().unwrap();

        // return cached value if it exists
        if let Some(image) = self.cache.get_bytes(&cache_key)? {
            return Ok(image);
        }

        // otherwise, request value from remote
        let response = self
            .io_pool
            .install(|| self.api.download_resource(&remote.access_token, &url));
        let bytes = response?;

        // remember result to cache
        self.cache.put_bytes(&cache_key, &bytes)?;
        // return result and release lock
        Ok(bytes.to_vec())
    }
}

/// Mapper from response to metadata
fn extract_metadata(values: &[Node]) -> RemoteMetadata {
    let mut queue = VecDeque::new();
    let mut name_to_node = HashMap::new();
    for value in values {
        queue.push_back(value);
    }
    while let Some(current) = queue.pop_front() {
        if !current.name.is_empty() && !name_to_node.contains_key(&current.name) {
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
    RemoteMetadata { name_to_node }
}
