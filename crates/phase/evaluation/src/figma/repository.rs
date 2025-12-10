use super::{Batched, Batcher, NodeMetadata};
use crate::{Error, Result};
use dashmap::DashMap;
use key_mutex::KeyMutex;
use lib_cache::{Cache, CacheKey};
use lib_figma_fluent::{FigmaApi, GetImageQueryParameters, GetImageResponse};
use log::{debug, warn};
use phase_loading::RemoteSource;
use retry::delay::Fixed;
use retry::retry_with_index;
use retry::{OperationResult, delay::jitter};
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use ureq::Error::Io;
use ureq::Error::StatusCode;

static FIGMA_500_NOTIFICATION: LazyLock<()> = LazyLock::new(
    || warn!(target: "FigmaRepository", "It looks like we DDoSed the Figma REST API — slowing down a bit..."),
);

#[derive(Clone)]
pub struct FigmaRepository {
    api: FigmaApi,
    batched_api: Arc<DashMap<BatchKey, ExportImgBatcher>>,
    cache: Cache,
    locks: KeyMutex<CacheKey, ()>,
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

pub type ExportImgBatcher = Batcher<String, BatchedApi, lib_figma_fluent::Result<GetImageResponse>>;

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
        }
    }

    pub fn export(
        &self,
        remote: &Arc<RemoteSource>,
        node: &NodeMetadata,
        format: &str,
        scale: f32,
        on_export_start: impl FnOnce(),
        on_cache_hit: impl FnOnce(),
    ) -> Result<DownloadUrl> {
        // construct unique cache key
        let cache_key = CacheKey::builder()
            .set_tag(Self::EXPORTED_IMAGE_TAG)
            .write_str(&remote.file_key)
            .write_str(&node.id)
            .write_u64(node.hash)
            .write_str(format)
            .write_str(&scale.to_string())
            .build();

        // return cached value if it exists
        if let Some(url) = self.cache.get::<DownloadUrl>(&cache_key)? {
            on_cache_hit();
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

        let response = retry_with_index(Fixed::from_millis(5000).map(jitter), |attempt| {
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
                    if no_requested_node_attempts.load(Ordering::SeqCst) < 5 {
                        OperationResult::Retry(err)
                    } else {
                        OperationResult::Err(err)
                    }
                }
                Ok(result) => OperationResult::Ok(result.to_owned()),
                Err(e) => match e {
                    lib_figma_fluent::Error::RateLimit {
                        retry_after_sec,
                        figma_plan_tier,
                        figma_limit_type,
                    } => {
                        warn!(target: "RateLimit", "{retry_after_sec}s, {figma_plan_tier}, {figma_limit_type}");
                        OperationResult::Err(Error::ExportImage(e.to_string()))
                    }
                    lib_figma_fluent::Error::Ureq(e) => match &e {
                        StatusCode(500..=599) => {
                            debug!(target: "FigmaRepository", "figma server error: {e}");
                            let _ = &*FIGMA_500_NOTIFICATION;
                            OperationResult::Retry(Error::ExportImage(e.to_string()))
                        }
                        Io(err) if matches!(err.kind(), std::io::ErrorKind::UnexpectedEof) => {
                            debug!(target: "FigmaRepository", "figma disconnected: {e}");
                            let _ = &*FIGMA_500_NOTIFICATION;
                            OperationResult::Retry(Error::ExportImage(e.to_string()))
                        }
                        _ => OperationResult::Err(Error::ExportImage(e.to_string())),
                    },
                },
            }
        });

        let url = {
            let response = match response.as_ref() {
                Ok(response) => response,
                Err(e) => return Err(Error::ExportImage(e.to_string())),
            };
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
        let response = retry_with_index(Fixed::from_millis(250).map(jitter), |_| {
            match self.api.download_resource(&remote.access_token, url) {
                Ok(value) => OperationResult::Ok(value),
                Err(e) => match &e {
                    lib_figma_fluent::Error::RateLimit {
                        retry_after_sec: _,
                        figma_plan_tier: _,
                        figma_limit_type: _,
                    } => OperationResult::Retry(Error::ExportImage(e.to_string())),
                    lib_figma_fluent::Error::Ureq(e) => match e {
                        StatusCode(500..=599) => {
                            debug!(target: "FigmaRepository", "figma server error: {e}");
                            let _ = &*FIGMA_500_NOTIFICATION;
                            OperationResult::Retry(Error::ExportImage(e.to_string()))
                        }
                        Io(err) if matches!(err.kind(), std::io::ErrorKind::UnexpectedEof) => {
                            debug!(target: "FigmaRepository", "figma disconnected: {e}");
                            let _ = &*FIGMA_500_NOTIFICATION;
                            OperationResult::Retry(Error::ExportImage(e.to_string()))
                        }
                        _ => OperationResult::Err(Error::ExportImage(e.to_string())),
                    },
                },
            }
        });
        let bytes = response?;

        // remember result to cache
        self.cache.put_bytes(&cache_key, &bytes)?;
        // return result and release lock
        Ok(bytes.to_vec())
    }
}

impl Batched<String, lib_figma_fluent::Result<GetImageResponse>> for BatchedApi {
    fn execute(&self, ids: Vec<String>) -> lib_figma_fluent::Result<GetImageResponse> {
        let BatchedApi {
            api,
            remote,
            format,
            scale,
        } = self;
        debug!(target: "FigmaRepository", "Batched request: ids=[{}]; format={format}; scale={scale}", ids.join(","));
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
