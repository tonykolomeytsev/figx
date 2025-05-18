use std::{
    fs::File,
    hash::Hasher,
    io::{BufReader, Read},
    path::Path,
    time::UNIX_EPOCH,
};

use log::warn;

/// Generate a fingerprint for a file based on metadata
///
/// Creates a deterministic CacheKey for a file using its path, size, and last modified timestamp.
/// This method does not read the file’s contents but instead uses file metadata to generate
/// a unique fingerprint.
///
/// This approach is useful for cache invalidation scenarios where the file content is assumed
/// to change if its metadata (like size or timestamp) changes. This approach is much faster than
/// calculating digest from file content.
///
/// # Example
///
/// ```
/// # use std::path::Path;
/// # use phase_evaluation::get_file_fingerprint;
/// let path = Path::new("hashing.rs");
/// match get_file_fingerprint(path) {
///     Ok(key) => println!("File fingerprint: {:?}", key),
///     Err(err) => eprintln!("Failed to generate fingerprint: {}", err),
/// }
/// ```
pub fn get_file_fingerprint(path: &Path) -> std::io::Result<u64> {
    let metadata = path.metadata()?;
    let last_modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| {
            warn!(
                "Unable to unwrap last_modified metadata field for file {}. Falling back to default value. Cause: {e}",
                path.display()
            );
            Default::default()
        })
        .as_millis();
    // Generate CacheKey for this file
    let mut hasher = xxhash_rust::xxh64::Xxh64::default();
    hasher.write(path.to_string_lossy().as_bytes());
    hasher.write_u64(metadata.len());
    hasher.write_u128(last_modified);
    Ok(hasher.finish())
}

/// Generate a content-based digest for a file
///
/// Reads the entire contents of a file and generates a CacheKey by hashing its bytes.
/// Unlike [`get_file_fingerprint`], this method ensures that even metadata-invisible
/// changes (e.g., content modified without changing file size or timestamp) are captured.
///
/// This is ideal when precise change detection is required, regardless of file metadata.
/// But this approach is much slower than calculating fingerprint from metadata.
///
/// # Example
///
/// ```
/// # use std::path::Path;
/// # use phase_evaluation::get_file_digest;
/// let path = Path::new("hashing.rs");
/// match get_file_digest(path) {
///     Ok(key) => println!("File digest: {:?}", key),
///     Err(err) => eprintln!("Failed to compute digest: {}", err),
/// }
/// ```
pub fn get_file_digest(path: &Path) -> std::io::Result<u64> {
    let input = File::open(path)?;
    let mut reader = BufReader::new(input);
    let mut hasher = xxhash_rust::xxh64::Xxh64::default();
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.write(&buffer[..count]);
    }
    Ok(hasher.finish())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {

    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn calculating_fingerprint_of_existing_file__EXPECT__ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("temp.txt");
        let _ = File::create(&file_path).unwrap();

        let fingerprint = get_file_fingerprint(&file_path);
        assert!(fingerprint.is_ok())
    }

    #[test]
    fn calculating_digest_of_existing_file__EXPECT__ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("temp.txt");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "Hello world!").unwrap();

        let fingerprint = get_file_digest(&file_path).unwrap();
        assert_eq!("9157857784689950130", format!("{:?}", fingerprint));
    }
}
