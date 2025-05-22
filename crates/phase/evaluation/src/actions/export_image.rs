use log::debug;
use phase_loading::RemoteSource;

use crate::{
    EvalContext, Result,
    figma::{DownloadUrl, NodeMetadata},
};

pub fn export_image(ctx: &EvalContext, args: ExportImageArgs, on_export_start: impl FnOnce()) -> Result<DownloadUrl> {
    debug!("exporting {}: {} ({})", args.format, args.node.id, args.node.name);
    ctx.figma_repository
        .export(args.remote, args.node, args.format, args.scale, on_export_start)
}

pub struct ExportImageArgs<'a> {
    pub remote: &'a RemoteSource,
    pub node: &'a NodeMetadata,
    pub format: &'a str,
    pub scale: f32,
}
