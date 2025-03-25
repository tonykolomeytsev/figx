use super::ExecutionContextExt;
use super::volatile_action;
use crate::Error;
use crate::EvalState;
use crate::Result;
use lib_cache::CacheKey;
use lib_figma::GetImageQueryParameters;
use lib_graph_exec::action::Action;
use lib_graph_exec::action::ActionDiagnostics;
use lib_graph_exec::action::ExecutionContext;
use lib_label::Label;
use log::debug;
use phase_loading::RemoteSource;
use std::sync::Arc;

pub struct ExportImgAction {
    pub label: Label,
    pub remote: Arc<RemoteSource>,
    pub format: String,
    pub scale: f32,
}

impl Action<CacheKey, Error, EvalState> for ExportImgAction {
    fn execute(&self, ctx: ExecutionContext<CacheKey, EvalState>) -> Result<CacheKey> {
        let find_node_cache_key = ctx.single_input()?;
        let stable_cache_key = CacheKey::builder()
            .write(find_node_cache_key.as_ref())
            .write_str(&self.format)
            .write_str(&self.scale.to_string())
            .build();

        volatile_action(&ctx.state.cache, stable_cache_key, false, |cache_key| {
            self.export_img_impl(find_node_cache_key, cache_key, &ctx.state)
        })
    }

    fn diagnostics_info(&self) -> ActionDiagnostics {
        ActionDiagnostics {
            name: "Export image from Figma".to_string(),
            params: vec![
                ("format".to_string(), self.format.clone()),
                ("scale".to_string(), self.scale.to_string()),
            ],
        }
    }
}

impl ExportImgAction {
    pub const TAG: u8 = 0x02;

    fn export_img_impl(
        &self,
        find_node_cache_key: &CacheKey,
        stable_cache_key: CacheKey,
        state: &EvalState,
    ) -> Result<CacheKey> {
        let ExportImgAction {
            label,
            remote,
            format,
            scale,
        } = &self;

        let ui_state = state.renderer.get_handle();
        ui_state.set_state(lib_pretty::State::Exporting(label.to_string()));

        let node_id = state.cache.require::<String>(find_node_cache_key)?;
        debug!("export {format} with scale {scale} from node with id '{node_id}'");
        let export_response = state.figma_api.get_image(
            &remote.access_token,
            &remote.file_key,
            GetImageQueryParameters {
                ids: Some(vec![node_id.to_string()]),
                scale: Some(*scale),
                format: Some(format.to_string()),
                ..Default::default()
            },
        )?;

        if let Some(error) = export_response.err {
            return Err(Error::ExportImage(format!(
                "got response with error: {error}"
            )));
        }

        let download_url = match export_response.images.get(&node_id) {
            Some(url) => url,
            None => {
                return Err(Error::ExportImage(format!(
                    "response has no requested node with id '{node_id}'"
                )));
            }
        };
        let download_url = match download_url {
            Some(url) => url,
            None => {
                return Err(Error::ExportImage(format!(
                    "requested node with id '{node_id}' was not rendered by Figma backend",
                )));
            }
        };
        let volatile_cache_key = CacheKey::builder()
            .set_tag(Self::TAG)
            .write(stable_cache_key.as_ref())
            .write_str(download_url)
            .build();

        state.cache.put(&volatile_cache_key, &download_url)?;
        Ok(volatile_cache_key)
    }
}
