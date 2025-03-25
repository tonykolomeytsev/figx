use std::io::Cursor;

use lib_cache::CacheKey;
use lib_graph_exec::action::{Action, ActionDiagnostics, ExecutionContext};
use log::debug;

use crate::{Error, EvalState, Result};

use super::{ExecutionContextExt, stable_action};

pub struct ScalePngAction {
    pub factor: f32,
    pub density: String,
}

impl Action<CacheKey, Error, EvalState> for ScalePngAction {
    fn execute(&self, ctx: ExecutionContext<CacheKey, EvalState>) -> Result<CacheKey> {
        let download_img_cache_key = ctx.single_input()?;
        let stable_cache_key = CacheKey::builder()
            .set_tag(Self::TAG)
            .write(download_img_cache_key.as_ref())
            .write_str(&self.factor.to_string())
            .build();

        stable_action(&ctx.state.cache, stable_cache_key, false, |cache_key| {
            self.scale_png_impl(download_img_cache_key, cache_key, &ctx.state)
        })
    }

    fn diagnostics_info(&self) -> ActionDiagnostics {
        ActionDiagnostics {
            name: "Scale PNG".to_string(),
            params: vec![
                ("density".to_string(), self.density.clone()),
                ("factor".to_string(), self.factor.to_string()),
            ],
        }
    }
}

impl ScalePngAction {
    pub const TAG: u8 = 0x07;

    fn scale_png_impl(
        &self,
        download_img_cache_key: &CacheKey,
        stable_cache_key: CacheKey,
        state: &EvalState,
    ) -> Result<()> {
        let ScalePngAction { factor, .. } = self;
        debug!("Scaling image: {download_img_cache_key:?} => {stable_cache_key:?}");
        let png_bytes = state.cache.require_bytes(download_img_cache_key)?;
        let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png)?;
        // resize img if needed
        let img = match *factor {
            1.0 => img,
            scale => {
                let new_width = (img.width() as f32 * scale).round() as u32;
                let new_height = (img.height() as f32 * scale).round() as u32;
                img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
            }
        };

        let mut buf = Vec::new();
        img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;
        state.cache.put_slice(&stable_cache_key, &buf)?;
        Ok(())
    }
}
