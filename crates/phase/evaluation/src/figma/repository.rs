use super::{Batched, Batcher, NodeMetadata, RemoteMetadata};
use crate::{Error, Result};
use dashmap::DashMap;
use key_mutex::KeyMutex;
use lib_cache::{Cache, CacheKey};
use lib_figma::{
    FigmaApi, GetFileNodesQueryParameters, GetImageQueryParameters, GetImageResponse, Node,
};
use log::{debug, warn};
use phase_loading::RemoteSource;
use retry::delay::Exponential;
use retry::retry_with_index;
use retry::{OperationResult, delay::jitter};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use std::{
    collections::{HashMap, VecDeque},
    sync::LazyLock,
};
use ureq::Error::StatusCode;

static RATE_LIMIT_NOTIFICATION: LazyLock<()> = LazyLock::new(
    || warn!(target: "FigmaRepository", "REST API rate limit has been hit. Subsequent requests will be throttled."),
);

#[derive(Clone)]
pub struct FigmaRepository {
    api: FigmaApi,
    batched_api: Arc<DashMap<BatchKey, ExportImgBatcher>>,
    cache: Cache,
    locks: KeyMutex<CacheKey, ()>,
    refetch_done: Arc<AtomicBool>,
}

pub struct BatchedApi {
    api: FigmaApi,
    remote: Arc<RemoteSource>,
    format: String,
    scale: f32,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct BatchKey(String);

impl BatchKey {
    pub fn from(file_key: &str, format: &str, scale: f32) -> Self {
        Self(format!("{file_key}:{format}:{scale}"))
    }
}

pub type ExportImgBatcher = Batcher<String, BatchedApi, lib_figma::Result<GetImageResponse>>;

pub type DownloadUrl = String;

impl FigmaRepository {
    pub const REMOTE_SOURCE_TAG: u8 = 0x42;
    pub const EXPORTED_IMAGE_TAG: u8 = 0x43;
    pub const DOWNLOADED_IMAGE_TAG: u8 = 0x44;

    pub fn new(api: FigmaApi, cache: Cache) -> Self {
        Self {
            api,
            batched_api: Arc::new(DashMap::new()),
            cache,
            locks: KeyMutex::new(),
            refetch_done: Arc::new(AtomicBool::new(false)),
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

        // return cached value if it exists
        if !refetch {
            if let Some(metadata) = self.cache.get::<RemoteMetadata>(&cache_key)? {
                return Ok(metadata);
            }
        }

        // this section will be accessed by only one thread for one remote
        let _lock = self.locks.lock(cache_key.clone()).unwrap();

        // check if refetch really needed
        let refetch = refetch && !self.refetch_done.swap(true, Ordering::SeqCst);

        // return cached value if it exists
        if !refetch {
            if let Some(metadata) = self.cache.get::<RemoteMetadata>(&cache_key)? {
                return Ok(metadata);
            }
        }

        // otherwise, request value from remote
        on_fetch_start();
        let response = self.api.get_file_nodes(
            &remote.access_token,
            &remote.file_key,
            GetFileNodesQueryParameters {
                ids: Some(&remote.container_node_ids),
                geometry: Some("paths"),
                ..Default::default()
            },
        )?;

        let metadata = {
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
        remote: &Arc<RemoteSource>,
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

        // return cached value if it exists
        if let Some(url) = self.cache.get::<DownloadUrl>(&cache_key)? {
            return Ok(url);
        }

        // this section will be accessed by only one thread for one node
        let _lock = self.locks.lock(cache_key.clone()).unwrap();

        // return cached value if it exists
        if let Some(url) = self.cache.get::<DownloadUrl>(&cache_key)? {
            return Ok(url);
        }

        // otherwise, request value from remote
        on_export_start();
        let batch_key = BatchKey::from(&remote.file_key, &format, scale);

        // Avoid DashMap's entry locking
        if let None = self.batched_api.get(&batch_key) {
            // Build batcher outside DashMap lock
            let new_batcher = Batcher::new(
                100,
                Duration::from_millis(1000),
                BatchedApi {
                    api: self.api.clone(),
                    remote: remote.clone(),
                    format: format.to_owned(),
                    scale: scale,
                },
            );
            self.batched_api.insert(batch_key.clone(), new_batcher);
        }
        // Then call batch
        let node_id = node.id.as_str();
        let node_name = node.name.as_str();
        let batched_api = self
            .batched_api
            .get(&batch_key)
            .expect("Value always exists");
        let no_requested_node_attempts = Arc::new(AtomicUsize::new(0));
        let response = retry_with_index(
            Exponential::from_millis_with_factor(5000, 2.0).map(jitter),
            |attempt| {
                if attempt > 1 {
                    debug!(target: "FigmaRepository" ,"retrying request: attempt #{}", attempt - 1);
                };
                match batched_api.batch(node.id.to_owned()).as_ref() {
                    Ok(result) if !result.images.contains_key(node_id) => {
                        debug!(target: "FigmaRepository", "response has no requested node '{node_name}' with id '{node_id}'");
                        no_requested_node_attempts.fetch_add(1, Ordering::SeqCst);
                        let err = Error::ExportImage(format!(
                            "response has no requested node '{node_name}' with id '{node_id}'",
                        ));
                        if no_requested_node_attempts.load(Ordering::SeqCst) < 3 {
                            OperationResult::Retry(err)
                        } else {
                            OperationResult::Err(err)
                        }
                    }
                    Ok(result) => OperationResult::Ok(result.to_owned()),
                    Err(e) => match e.0 {
                        StatusCode(code) if code == 429 => {
                            debug!(target: "FigmaRepository", "rate limit encountered");
                            let _ = &*RATE_LIMIT_NOTIFICATION;
                            OperationResult::Retry(Error::ExportImage(e.to_string()))
                        }
                        _ => OperationResult::Err(Error::ExportImage(e.to_string())),
                    },
                }
            },
        );

        let url = {
            let response = match response.as_ref() {
                Ok(response) => response,
                Err(e) => return Err(Error::ExportImage(e.to_string())),
            };
            if let Some(error) = &response.err {
                return Err(Error::ExportImage(format!(
                    "got response with error while exporting '{node_name}': {error}"
                )));
            }
            let download_url = match response.images.get(node_id) {
                Some(url) => url,
                None => {
                    return Err(Error::ExportImage(format!(
                        "response has no requested node '{node_name}' with id '{node_id}'",
                    )));
                }
            };
            match download_url {
                Some(url) => url,
                None => {
                    return Err(Error::ExportImage(format!(
                        "requested node '{node_name}' with id '{node_id}' was not rendered by Figma backend",
                    )));
                }
            }
        };

        // remember result to cache
        self.cache.put::<DownloadUrl>(&cache_key, &url)?;
        // return result and release lock
        Ok(url.to_owned())
    }

    pub fn download(&self, remote: &RemoteSource, url: &str) -> Result<Vec<u8>> {
        // construct unique cache key
        let cache_key = CacheKey::builder()
            .set_tag(Self::DOWNLOADED_IMAGE_TAG)
            .write_str(url)
            .build();

        // return cached value if it exists
        if let Some(image) = self.cache.get_bytes(&cache_key)? {
            return Ok(image);
        }

        // this section will be accessed by only one thread for one node
        let _lock = self.locks.lock(cache_key.clone()).unwrap();

        // return cached value if it exists
        if let Some(image) = self.cache.get_bytes(&cache_key)? {
            return Ok(image);
        }

        // otherwise, request value from remote
        let response = self.api.download_resource(&remote.access_token, url);
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
                    uses_raster_paints: !uses_only_vector_paints(current),
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

impl Batched<String, lib_figma::Result<GetImageResponse>> for BatchedApi {
    fn execute(&self, ids: Vec<String>) -> lib_figma::Result<GetImageResponse> {
        let BatchedApi {
            api,
            remote,
            format,
            scale,
        } = self;
        debug!(target: "FigmaRepository", "Batched request: ids={}; format={format}; scale={scale}", ids.join(","));
        Ok(api.get_image(
            &remote.access_token,
            &remote.file_key,
            GetImageQueryParameters {
                ids: Some(&ids),
                scale: Some(*scale),
                format: Some(format),
                ..Default::default()
            },
        )?)
    }
}

fn uses_only_vector_paints(node: &Node) -> bool {
    node.fills.iter().all(|it| it.r#type != "IMAGE")
        && node.children.iter().all(uses_only_vector_paints)
}
