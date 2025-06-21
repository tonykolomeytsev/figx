use crate::{EvalContext, Result, get_file_digest, get_file_fingerprint};
use bincode::{Decode, Encode};
use lib_cache::CacheKey;
use log::debug;
use std::path::Path;

const FILE_DIGEST_TAG: u8 = 0x01;

pub fn materialize(
    ctx: &EvalContext,
    args: MaterializeArgs,
    on_execute: impl FnOnce(),
) -> Result<()> {
    // construct unique cache key
    let cache_key = CacheKey::builder()
        .set_tag(FILE_DIGEST_TAG)
        .write(args.bytes)
        .write_str(args.file_extension)
        .write_str(args.file_name)
        .write(args.output_dir.to_string_lossy().as_bytes())
        .build();

    let output_file = args
        .output_dir
        .join(args.file_name)
        .with_extension(args.file_extension);

    // check if file already materialized
    if output_file.exists() {
        let cached_file_metadata = ctx.cache.get::<FileMetadata>(&cache_key)?;

        // firstly check fingerprint
        let actual_file_fingerprint = get_file_fingerprint(&output_file)?;
        match (&cached_file_metadata, actual_file_fingerprint) {
            (Some(cached), actual) if cached.fingerprint == actual => return Ok(()),
            _ => (),
        }

        // next check digest
        let actual_file_digest = get_file_digest(&output_file)?;
        match (&cached_file_metadata, actual_file_digest) {
            (Some(cached), actual) if cached.digest == actual => return Ok(()),
            _ => (),
        }
    }

    on_execute();
    debug!(target: "Materialize", "{}", output_file.display());
    std::fs::create_dir_all(args.output_dir)?;
    std::fs::write(&output_file, args.bytes)?;

    // remember file digest
    ctx.cache.put::<FileMetadata>(
        &cache_key,
        &FileMetadata {
            fingerprint: get_file_fingerprint(&output_file)?,
            digest: get_file_digest(&output_file)?,
        },
    )?;
    Ok(())
}

#[derive(Encode, Decode)]
struct FileMetadata {
    pub fingerprint: u64,
    pub digest: u64,
}

pub struct MaterializeArgs<'a> {
    pub output_dir: &'a Path,
    pub file_name: &'a str,
    pub file_extension: &'a str,
    pub bytes: &'a [u8],
}
