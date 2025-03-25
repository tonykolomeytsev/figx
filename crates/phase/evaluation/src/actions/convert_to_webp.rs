use super::ExecutionContextExt;
use super::stable_action;
use crate::Error;
use crate::EvalState;
use crate::Result;
use image::EncodableLayout;
use lib_cache::CacheKey;
use lib_graph_exec::action::Action;
use lib_graph_exec::action::ActionDiagnostics;
use lib_graph_exec::action::ExecutionContext;
use log::debug;

pub struct ConvertToWebpAction {
    pub quality: f32,
}

impl Action<CacheKey, Error, EvalState> for ConvertToWebpAction {
    fn execute(&self, ctx: ExecutionContext<CacheKey, EvalState>) -> Result<CacheKey> {
        let download_img_cache_key = ctx.single_input()?;
        let stable_cache_key = CacheKey::builder()
            .set_tag(Self::TAG)
            .write(download_img_cache_key.as_ref())
            .write_str(&self.quality.to_string())
            .build();

        stable_action(&ctx.state.cache, stable_cache_key, false, |cache_key| {
            self.convert_to_webp_impl(download_img_cache_key, cache_key, &ctx.state)
        })
    }

    fn diagnostics_info(&self) -> ActionDiagnostics {
        ActionDiagnostics {
            name: "Convert PNG to WEBP".to_string(),
            params: vec![("quality".to_string(), self.quality.to_string())],
        }
    }
}

impl ConvertToWebpAction {
    pub const TAG: u8 = 0x04;

    fn convert_to_webp_impl(
        &self,
        download_img_cache_key: &CacheKey,
        stable_cache_key: CacheKey,
        state: &EvalState,
    ) -> Result<()> {
        let ConvertToWebpAction { quality } = self;

        debug!("Transforming PNG to WEBP: {download_img_cache_key:?} => {stable_cache_key:?}");
        let png_bytes = state.cache.require_bytes(download_img_cache_key)?;
        let img = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png)?;
        let encoder = webp::Encoder::from_image(&img).map_err(|_| Error::WebpCreate)?; // fails if img is not RBG8 or RBGA8
        let webp = encoder.encode(*quality);
        state.cache.put_slice(&stable_cache_key, webp.as_bytes())?;

        Ok(())
    }
}
