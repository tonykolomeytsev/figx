use std::sync::Arc;

use crate::{
    EvalContext, Result,
    figma::{DownloadUrl, NodeMetadata},
};
use phase_loading::RemoteSource;

pub fn export_image(
    ctx: &EvalContext,
    args: ExportImageArgs,
    on_export_start: impl FnOnce(),
    on_cache_hit: impl FnOnce(),
) -> Result<DownloadUrl> {
    ctx.figma_repository.export(
        args.remote,
        args.node,
        args.format,
        args.scale,
        on_export_start,
        on_cache_hit,
    )
}

pub struct ExportImageArgs<'a> {
    pub remote: &'a Arc<RemoteSource>,
    pub node: &'a NodeMetadata,
    pub format: &'a str,
    pub scale: f32,
}
