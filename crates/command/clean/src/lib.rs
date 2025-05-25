mod error;
pub use error::*;
use phase_evaluation::{figma::FigmaRepository, setup_cache};
use phase_loading::load_invocation_context;

pub struct FeatureCleanOptions {
    pub all: bool,
}

pub fn clean(opts: FeatureCleanOptions) -> Result<()> {
    let ctx = load_invocation_context()?;
    let cache_dir = ctx.cache_dir;
    match opts {
        FeatureCleanOptions { all: true } => {
            let _ = std::fs::remove_dir_all(cache_dir);
        }
        FeatureCleanOptions { all: false } => {
            let cache = setup_cache(&cache_dir)?;
            let _ = cache.retain(|tag| match tag {
                FigmaRepository::REMOTE_SOURCE_TAG
                | FigmaRepository::DOWNLOADED_IMAGE_TAG
                | FigmaRepository::EXPORTED_IMAGE_TAG => true,
                _ => false,
            });
        }
    }
    Ok(())
}
