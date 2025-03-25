use lib_cache::CacheKey;
use lib_graph_exec::action::{Action, ActionDiagnostics, ExecutionContext};
use lib_svg2compose::SvgToComposeOptions;
use log::debug;

use crate::{Error, EvalState, Result};

use super::{DownloadImgAction, ExecutionContextExt, GetKotlinPackageAction, stable_action};

pub struct SvgToComposeAction {
    pub image_name: String,
    pub kotlin_explicit_api: bool,
}

impl Action<CacheKey, Error, EvalState> for SvgToComposeAction {
    fn execute(&self, ctx: ExecutionContext<CacheKey, EvalState>) -> Result<CacheKey> {
        let download_img_cache_key = ctx.tagged_input(DownloadImgAction::TAG)?;
        let kotlin_package_cache_key = ctx.tagged_input(GetKotlinPackageAction::TAG)?;
        let stable_cache_key = CacheKey::builder()
            .set_tag(Self::TAG)
            .write(download_img_cache_key.as_ref())
            .write(kotlin_package_cache_key.as_ref())
            .write_str(&self.image_name)
            .write_bool(self.kotlin_explicit_api)
            .build();

        stable_action(&ctx.state.cache, stable_cache_key, true, |cache_key| {
            self.svg_to_compose_impl(
                download_img_cache_key,
                kotlin_package_cache_key,
                cache_key,
                &ctx.state,
            )
        })
    }

    fn diagnostics_info(&self) -> ActionDiagnostics {
        ActionDiagnostics {
            name: "Convert SVG to Compose".to_string(),
            params: vec![
                ("image_name".to_string(), self.image_name.clone()),
                (
                    "kotlin_explicit_api".to_string(),
                    self.kotlin_explicit_api.to_string(),
                ),
            ],
        }
    }
}

impl SvgToComposeAction {
    pub const TAG: u8 = 0x05;

    fn svg_to_compose_impl(
        &self,
        download_img_cache_key: &CacheKey,
        kotlin_package_cache_key: &CacheKey,
        stable_cache_key: CacheKey,
        state: &EvalState,
    ) -> Result<()> {
        debug!("Transforming SVG to COMPOSE: {download_img_cache_key:?} => {stable_cache_key:?}");
        let package: String = state.cache.require(kotlin_package_cache_key)?;
        let svg_bytes = state.cache.require_bytes(download_img_cache_key)?;
        let compose = lib_svg2compose::transform_svg_to_compose(
            &svg_bytes,
            SvgToComposeOptions {
                image_name: self.image_name.to_owned(),
                package: if package.is_empty() {
                    state
                        .cache
                        .get::<String>(kotlin_package_cache_key)?
                        .unwrap_or_default()
                } else {
                    package
                },
                kotlin_explicit_api: self.kotlin_explicit_api,
            },
        )?;
        state.cache.put_slice(&stable_cache_key, &compose)?;

        Ok(())
    }
}
