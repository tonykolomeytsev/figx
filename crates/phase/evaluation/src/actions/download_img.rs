use std::sync::Arc;

use super::volatile_action;
use crate::Error;
use crate::EvalState;
use crate::ExecutionContextExt;
use crate::Result;
use lib_cache::CacheKey;
use lib_graph_exec::action::Action;
use lib_graph_exec::action::ActionDiagnostics;
use lib_graph_exec::action::ExecutionContext;
use lib_label::Label;
use log::debug;
use phase_loading::RemoteSource;

pub struct DownloadImgAction {
    pub label: Label,
    pub remote: Arc<RemoteSource>,
}

impl Action<CacheKey, Error, EvalState> for DownloadImgAction {
    fn execute(&self, ctx: ExecutionContext<CacheKey, EvalState>) -> Result<CacheKey> {
        let export_img_cache_key = ctx.single_input()?;
        let stable_cache_key = CacheKey::builder()
            .set_tag(Self::TAG)
            .write(export_img_cache_key.as_ref())
            .build();

        volatile_action(&ctx.state.cache, stable_cache_key, false, |cache_key| {
            self.download_img_impl(export_img_cache_key, cache_key, &ctx.state)
        })
    }

    fn diagnostics_info(&self) -> ActionDiagnostics {
        ActionDiagnostics {
            name: "Download image".to_string(),
            params: Vec::new(),
        }
    }
}

impl DownloadImgAction {
    pub const TAG: u8 = 0x03;

    fn download_img_impl(
        &self,
        export_img_cache_key: &CacheKey,
        stable_cache_key: CacheKey,
        state: &EvalState,
    ) -> Result<CacheKey> {
        let DownloadImgAction { label, remote } = &self;

        let ui_state = state.renderer.get_handle();
        ui_state.set_state(lib_pretty::State::Downloading(label.to_string()));

        let download_url: String = state.cache.require(export_img_cache_key)?;
        debug!("download image from remote: {download_url}");

        let image_bytes = state
            .figma_api
            .download_resource(&remote.access_token, &download_url)?;
        let volatile_cache_key = CacheKey::builder()
            .set_tag(Self::TAG)
            .write(stable_cache_key.as_ref())
            .write(image_bytes.as_ref())
            .build();

        state.cache.put_bytes(&volatile_cache_key, &image_bytes)?;
        Ok(volatile_cache_key)
    }
}
