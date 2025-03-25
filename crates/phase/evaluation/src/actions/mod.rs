mod convert_to_webp;
mod download_img;
mod export_img;
mod fetch_remote;
mod find_node;
mod get_kotlin_package;
mod materialize_img;
mod no_op;
mod svg_to_compose;
mod scale_png;

use crate::{EvalState, Result, get_file_digest, get_file_fingerprint};
pub use convert_to_webp::*;
pub use download_img::*;
pub use export_img::*;
pub use fetch_remote::*;
pub use find_node::*;
pub use get_kotlin_package::*;
use lib_cache::{Cache, CacheKey};
use lib_graph_exec::action::ExecutionContext;
use log::{debug, trace};
pub use materialize_img::*;
pub use no_op::*;
use phase_loading::RemoteSource;
use std::path::Path;
pub use svg_to_compose::*;
pub use scale_png::*;

/// Runs a volatile action with persistent caching based on a stable key.
///
/// In an offline-first system where results of "volatile" actions are cached until explicitly invalidated,
/// this function helps store and retrieve such results efficiently.
///
/// If a volatile result is already associated with the given `stable_cache_key`, it is returned directly.
/// Otherwise, the provided action is executed to compute a new volatile cache key, which is then cached
/// under the stable key for future use.
///
/// # Returns
///
/// - `Ok(CacheKey)`: The cached or newly computed volatile cache key.
/// - `Err(...)`: If an error occurs when accessing the cache or during execution of the action.
///
/// # Side Effects
///
/// This function creates a mapping in the cache from the `stable_cache_key` to the computed `volatile_cache_key`.
/// The actual data is expected to be stored under the `volatile_cache_key`, while the `stable_cache_key`
/// serves as an indirection layer to look up the current version of the result.
/// This enables the system to treat volatile actions as if they were cacheable, while preserving
/// the ability to update them explicitly.
///
/// # Use Case
///
/// This is commonly used in build systems or task runners, where volatile actions (e.g., accessing external state,
/// reading time-dependent input) need to be memoized for consistency across offline or incremental runs,
/// but are not purely deterministic based on inputs.
///
/// # Example
///
/// ```ignore
/// let final_key = volatile_action(&cache, &stable_key, false, || {
///     // Perform side-effecting or unstable operation
/// })?;
/// ```
pub fn volatile_action(
    cache: &Cache,
    stable_cache_key: CacheKey,
    force_rerun: bool,
    action_impl: impl Fn(CacheKey) -> Result<CacheKey>,
) -> Result<CacheKey> {
    match (force_rerun, cache.get(&stable_cache_key)?) {
        (true, _) | (false, None) => {
            if force_rerun {
                trace!("Executing action because force_rerun = true")
            } else {
                trace!(
                    "Executing action because no volatile key found for stable key {stable_cache_key:?}"
                );
            }
            let volatile_cache_key = action_impl(stable_cache_key.clone())?;
            cache.put(&stable_cache_key, &volatile_cache_key)?;
            trace!("Created mapping in the cache: {stable_cache_key:?} => {volatile_cache_key:?}");
            Ok(volatile_cache_key)
        }
        (_, Some(volatile_cache_key)) => {
            trace!("Got volatile key from cache: {stable_cache_key:?} => {volatile_cache_key:?}");
            Ok(volatile_cache_key)
        }
    }
}

/// Runs a deterministic action and returns its stable cache key.
///
/// This function is used for actions whose results are purely determined by their inputs,
/// and therefore can be reliably cached using a single stable key. If the result is already present
/// in the cache under the given `stable_cache_key`, the action is skipped. Otherwise, the provided
/// action is executed, and it is expected to populate the cache under that key.
///
/// # Returns
///
/// - `Ok(CacheKey)`: Always returns the `stable_cache_key`, either because it was already cached,
///   or because the action was successfully executed and the cache was populated.
/// - `Err(...)`: If an error occurs when accessing the cache or during execution of the action.
///
/// # Side Effects
///
/// The provided `action_impl` is expected to write the result to the cache using the given `stable_cache_key`.
/// This function itself does **not** store any data in the cache — it only ensures the action was executed
/// if the key wasn't already present.
///
/// # Use Case
///
/// Useful in systems with deterministic, cacheable operations where results do not vary across runs
/// with the same inputs — such as compilation, hashing, or code generation.
///
/// # Example
///
/// ```ignore
/// let key = stable_action(&cache, &stable_key, false, || {
///     // your actual action call
/// })?;
/// ```
pub fn stable_action(
    cache: &Cache,
    stable_cache_key: CacheKey,
    force_rerun: bool,
    action_impl: impl Fn(CacheKey) -> Result<()>,
) -> Result<CacheKey> {
    if force_rerun || !cache.contains_key(&stable_cache_key)? {
        action_impl(stable_cache_key.clone())?;
    }
    Ok(stable_cache_key)
}

/// Runs action and returns the same `stable_cache_key`.
pub fn fs_output_action(
    cache: &Cache,
    stable_cache_key: CacheKey,
    output_file: &Path,
    action_impl: impl Fn(CacheKey) -> Result<()>,
) -> Result<CacheKey> {
    let fingerprint_cache_key = CacheKey::builder()
        .write(stable_cache_key.as_ref())
        .write_str("fingerprint")
        .build();
    let digest_cache_key = CacheKey::builder()
        .write(stable_cache_key.as_ref())
        .write_str("digest")
        .build();

    if !output_file.exists() {
        action_impl(stable_cache_key.clone())?;
    } else {
        match (
            cache.get::<u64>(&fingerprint_cache_key)?,
            cache.get::<u64>(&digest_cache_key)?,
        ) {
            (Some(fingerprint), Some(digest)) => {
                if get_file_fingerprint(output_file)? == fingerprint {
                    debug!("{} fingerprint has not changed", output_file.display());
                    return Ok(stable_cache_key.clone());
                }
                if get_file_digest(output_file)? == digest {
                    cache
                        .put::<u64>(&fingerprint_cache_key, &get_file_fingerprint(output_file)?)?;
                    debug!("{} digest has not changed", output_file.display(),);
                    return Ok(stable_cache_key.clone());
                }
            }
            _ => (),
        }
        action_impl(stable_cache_key.clone())?;
    }

    cache.put::<u64>(&fingerprint_cache_key, &get_file_fingerprint(output_file)?)?;
    cache.put::<u64>(&digest_cache_key, &get_file_digest(output_file)?)?;
    Ok(stable_cache_key)
}

pub trait ToCacheKey {
    fn to_cache_key(&self) -> CacheKey;
}

impl ToCacheKey for RemoteSource {
    fn to_cache_key(&self) -> CacheKey {
        CacheKey::builder()
            .write_str(&self.file_key)
            .write_str(&self.container_node_ids.join(","))
            .build()
    }
}

pub trait ExecutionContextExt {
    fn single_input(&self) -> Result<&CacheKey>;
    fn tagged_input(&self, tag: u8) -> Result<&CacheKey>;
}

impl ExecutionContextExt for ExecutionContext<CacheKey, EvalState> {
    fn single_input(&self) -> Result<&CacheKey> {
        Ok(self
            .inputs
            .first()
            .ok_or(crate::Error::ActionSingleInputAbsent)?)
    }

    fn tagged_input(&self, tag: u8) -> Result<&CacheKey> {
        Ok(self
            .inputs
            .iter()
            .find(|it| it.tag() == tag)
            .ok_or(crate::Error::ActionTaggedInputAbsent)?)
    }
}
