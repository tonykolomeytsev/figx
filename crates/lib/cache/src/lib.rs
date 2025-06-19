use bincode::{Decode, Encode};
use bytes::Bytes;
pub use error::*;
pub use key::*;
use log::debug;
use std::{path::Path, sync::Arc};
use surrealkv::{IsolationLevel, Options, Store};

mod error;
mod key;

#[derive(Clone)]
pub struct Cache {
    store: Arc<Store>,
    config: Arc<CacheConfig>,
}

#[derive(Default)]
pub struct CacheConfig {
    /// If true, transaction write conflicts (same key) will be ignores
    pub ignore_write_conflict: bool,
    /// If true, then values ​​that cannot be deserialized will return None
    pub allow_deserialization_error: bool,
}

impl Cache {
    /// Creates a new cache instance with the specified directory for storage.
    ///
    /// # Arguments
    /// * `dir` - Directory path where data will be stored
    ///
    /// # Errors
    /// Returns `Err` if storage initialization fails or directory can't be accessed
    pub fn new(dir: impl AsRef<Path>, config: CacheConfig) -> Result<Self> {
        let mut opts = Options::new();
        opts.dir = dir.as_ref().into();

        // region: Storage configuration
        opts.disk_persistence = true;
        // Values smaller than this stored in memory
        opts.max_value_threshold = 4096;
        // Controls when new log segments are created, affects compaction frequency
        opts.max_segment_size = 268_435_456; // 256MB segment size
        // endregion

        // region: Transaction and versioning
        opts.isolation_level = IsolationLevel::SnapshotIsolation;
        opts.enable_versions = false;
        // endregion

        // region: Cache settings
        // Number of values that can be cached to avoid disk lookups
        opts.max_value_cache_size = 1000;
        // endregion

        let store = Arc::new(Store::new(opts).map_err(Error::initialization)?);
        Ok(Self {
            store,
            config: Arc::new(config),
        })
    }

    /// Stores the raw bytes `value` in the cache by `key`.
    pub fn put_bytes(&self, key: &CacheKey, value: &[u8]) -> Result<()> {
        let mut txn = self.store.begin()?;
        txn.set(key.as_ref(), value)?;
        use surrealkv::Error::*;
        match txn.commit() {
            Err(TransactionWriteConflict) if self.config.ignore_write_conflict => Ok(()),
            res => res,
        }?;
        Ok(())
    }

    /// Retrieves raw bytes from the cache by `key`.
    pub fn get_bytes(&self, key: &CacheKey) -> Result<Option<Vec<u8>>> {
        let mut txn = self.store.begin()?;
        Ok(txn.get(key.as_ref())?)
    }

    /// Removes the `key` and its associated `value` from the cache.
    pub fn delete(&self, key: &CacheKey) -> Result<()> {
        let mut txn = self.store.begin()?;
        txn.delete(key.as_ref())?;
        txn.commit()?;
        Ok(())
    }

    /// Checks if the cache contains the specified `key`.
    pub fn contains_key(&self, key: &CacheKey) -> Result<bool> {
        let mut txn = self.store.begin()?;
        Ok(txn.get(key.as_ref())?.is_some())
    }

    /// Serializes and stores the `value` in the cache with the given `key`.
    pub fn put<E>(&self, key: &CacheKey, value: &E) -> Result<()>
    where
        E: Encode,
    {
        let serialized_value = bincode::encode_to_vec(value, bincode::config::standard())
            .map_err(Error::deserialization)?;
        self.put_bytes(key, &Bytes::from(serialized_value))
    }

    /// Retrieves and deserializes a value from the cache by `key`.
    pub fn get<D>(&self, key: &CacheKey) -> Result<Option<D>>
    where
        D: Decode<()>,
    {
        if let Some(raw_value) = self.get_bytes(key)? {
            let (deserialized_value, _) =
                match bincode::decode_from_slice(&raw_value, bincode::config::standard()) {
                    Err(_) if self.config.allow_deserialization_error => return Ok(None),
                    res => res.map_err(Error::deserialization)?,
                };
            Ok(Some(deserialized_value))
        } else {
            Ok(None)
        }
    }

    pub fn require<D>(&self, key: &CacheKey) -> Result<D>
    where
        D: Decode<()>,
    {
        match self.get(key) {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(Error::MissingRequiredValue(format!("{key:?}"))),
            Err(e) => Err(e),
        }
    }

    pub fn require_bytes(&self, key: &CacheKey) -> Result<Vec<u8>> {
        match self.get_bytes(key) {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(Error::MissingRequiredValue(format!("{key:?}"))),
            Err(e) => Err(e),
        }
    }

    pub fn retain(&self, predicate: impl Fn(u8) -> bool) -> Result<()> {
        let txn = self.store.begin()?;
        let start: &[u8] = &[0x00; 9];
        let end: &[u8] = &[0xff; 9];
        let mut keys_to_delete = Vec::new();
        txn.keys(start..end, None)
            .filter(|arr| match arr.first() {
                Some(tag) => !predicate(*tag),
                None => false,
            })
            .for_each(|key| keys_to_delete.push(key));
        let mut txn = self.store.begin()?;
        for key in keys_to_delete {
            debug!(target: "Cache", "deleting cache key {:?}", key);
            txn.delete(key)?;
        }
        txn.commit()?;
        Ok(())
    }
}
