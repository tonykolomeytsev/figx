use std::sync::Arc;

use super::{
    download_image::DownloadImageArgs,
    export_image::{ExportImageArgs, export_image},
    fetch_remote::FetchRemoteArgs,
    find_node_by_name::FindNodeByNameArgs,
};
use crate::{
    EvalContext, Result,
    actions::{
        download_image::download_image, fetch_remote::fetch_remote,
        find_node_by_name::find_node_by_name,
    },
};
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
                    node: find_node_by_name(FindNodeByNameArgs {
                        name: args.node_name,
                        remote: &fetch_remote(
                            ctx,
                            FetchRemoteArgs {
                                remote: args.remote,
                            },
                            || info!(target: "Updating", "remote {} index", args.remote),
                        )?,
                    })?,
                },
                || {
                    info!(target: "Downloading", "{format} for `{label}`{variant} to file",
                        format = args.format.to_ascii_uppercase(),
                        label = args.label.fitted(50),
                        variant = if args.variant_name.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", args.variant_name)
                        }
                    )
                },
            )?,
        },
    )
}

pub struct GetRemoteImageArgs<'a> {
    pub label: &'a Label,
    pub remote: &'a Arc<RemoteSource>,
    pub node_name: &'a str,
    pub format: &'a str,
    pub scale: f32,
    pub variant_name: &'a str,
}
