use std::sync::Arc;

use super::{
    download_image::DownloadImageArgs,
    export_image::{ExportImageArgs, export_image},
};
use crate::{actions::download_image::download_image, figma::NodeMetadata, EvalContext, Result};
use lib_label::Label;
use log::info;
use phase_loading::RemoteSource;

/// Shortcut action
pub fn get_remote_image(ctx: &EvalContext, args: GetRemoteImageArgs) -> Result<Vec<u8>> {
    download_image(
        ctx,
        DownloadImageArgs {
            remote: args.remote,
            url: &export_image(
                ctx,
                ExportImageArgs {
                    remote: args.remote,
                    format: args.format,
                    scale: args.scale,
                    node: args.node,
                },
                || {
                    info!(target: "Downloading", "{format} for `{label}`{variant}",
                        format = args.format.to_ascii_uppercase(),
                        label = args.label.fitted(50),
                        variant = if args.variant_name.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", args.variant_name)
                        }
                    )
                },
                || ctx.metrics.targets_from_cache.increment(),
            )?,
        },
    )
}

pub struct GetRemoteImageArgs<'a> {
    pub label: &'a Label,
    pub remote: &'a Arc<RemoteSource>,
    pub node: &'a NodeMetadata,
    pub format: &'a str,
    pub scale: f32,
    pub variant_name: &'a str,
}
