use crate::Error;
use crate::EvalState;
use crate::Result;
use lib_cache::CacheKey;
use lib_graph_exec::action::Action;
use lib_graph_exec::action::ActionDiagnostics;
use lib_graph_exec::action::ExecutionContext;
use log::debug;
use std::path::Path;
use std::path::PathBuf;

use super::ExecutionContextExt;
use super::fs_output_action;

pub struct MaterializeImgAction {
    pub output_dir: PathBuf,
    pub image_name: String,
    pub extension: String,
}

impl Action<CacheKey, Error, EvalState> for MaterializeImgAction {
    fn execute(&self, ctx: ExecutionContext<CacheKey, EvalState>) -> Result<CacheKey> {
        let image_cache_key = ctx.single_input()?;
        let output_file = self
            .output_dir
            .join(&self.image_name)
            .with_extension(&self.extension);

        let stable_cache_key = CacheKey::builder()
            .write(image_cache_key.as_ref())
            .write(output_file.to_string_lossy().as_bytes())
            .build();

        fs_output_action(
            &ctx.state.cache,
            stable_cache_key,
            &output_file,
            |cache_key| {
                self.materialize_img_impl(image_cache_key, cache_key, &output_file, &ctx.state)
            },
        )
    }

    fn diagnostics_info(&self) -> ActionDiagnostics {
        ActionDiagnostics {
            name: "Materialize resource".to_string(),
            params: vec![
                ("image_name".to_string(), self.image_name.clone()),
                ("extension".to_string(), self.extension.clone()),
            ],
        }
    }
}

impl MaterializeImgAction {
    fn materialize_img_impl(
        &self,
        image_cache_key: &CacheKey,
        stable_cache_key: CacheKey,
        output_file: &Path,
        state: &EvalState,
    ) -> Result<()> {
        debug!("materializing image: {}", output_file.display());
        std::fs::create_dir_all(&self.output_dir)?;
        let image_bytes = state.cache.get_bytes(image_cache_key)?.unwrap();
        std::fs::write(output_file, image_bytes)?;
        state
            .cache
            .put(&stable_cache_key, &output_file.to_string_lossy().as_bytes())?;
        Ok(())
    }
}
