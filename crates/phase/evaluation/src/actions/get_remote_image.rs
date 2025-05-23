use lib_label::Label;
use log::info;
use phase_loading::RemoteSource;

use crate::{
    EvalContext, Result,
    actions::{
        download_image::download_image, fetch_remote::fetch_remote,
        find_node_by_name::find_node_by_name,
    },
};

use super::{
    download_image::DownloadImageArgs,
    export_image::{ExportImageArgs, export_image},
    fetch_remote::FetchRemoteArgs,
    find_node_by_name::FindNodeByNameArgs,
};

/// Shortcut action
pub fn get_remote_image(
    ctx: &EvalContext,
    args: GetRemoteImageArgs,
) -> Result<Vec<u8>> {
    download_image(
        ctx,
        DownloadImageArgs {
            remote: &args.remote,
            url: &export_image(
                ctx,
                ExportImageArgs {
                    remote: &args.remote,
                    format: args.format,
                    scale: args.scale,
                    node: find_node_by_name(FindNodeByNameArgs {
                        name: &args.node_name,
                        remote: &fetch_remote(
                            ctx,
                            FetchRemoteArgs {
                                remote: &args.remote,
                            },
                            || info!(target: "Fetching", "remote {}", args.remote),
                        )?,
                    })?,
                },
                || info!(target: "Downloading", "{} of scale {} for `{}`", args.format, args.scale, args.label.truncated_display(40)),
            )?,
        },
    )
}

pub struct GetRemoteImageArgs<'a> {
    pub label: &'a Label,
    pub remote: &'a RemoteSource,
    pub node_name: &'a str,
    pub format: &'a str,
    pub scale: f32,
}
