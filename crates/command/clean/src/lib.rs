mod error;
pub use error::*;
use log::info;
use phase_loading::load_invocation_context;

pub struct FeatureCleanOptions;

pub fn clean(_opts: FeatureCleanOptions) -> Result<()> {
    let ctx = load_invocation_context()?;
    let cache_dir = ctx.cache_dir;
    std::fs::remove_dir_all(cache_dir)?;
    info!("Cache directory has been cleared");
    Ok(())
}
