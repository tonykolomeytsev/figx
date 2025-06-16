use lib_label::Label;
use log::{info, warn};
use phase_loading::{RemoteSource, ResourceDiagnostics};

use crate::{
    EvalContext, Result,
    actions::{
        fetch_remote::{FetchRemoteArgs, fetch_remote},
        find_node_by_name::{FindNodeByNameArgs, find_node_by_name},
    },
    figma::NodeMetadata,
};

pub fn get_node<'a, 'b>(ctx: &'a EvalContext, args: GetNodeArgs<'a>) -> Result<NodeMetadata> {
    let remote = fetch_remote(
        ctx,
        FetchRemoteArgs {
            remote: args.remote,
        },
        || info!(target: "Updating", "remote index: {}", args.remote),
    )?;
    find_node_by_name(FindNodeByNameArgs {
        name: args.node_name,
        remote: &remote,
        diag: args.diag,
    })
    .cloned()
}

pub struct GetNodeArgs<'a> {
    pub node_name: &'a str,
    pub remote: &'a RemoteSource,
    pub diag: &'a ResourceDiagnostics,
}

pub fn ensure_is_vector_node(
    node: &NodeMetadata,
    node_name: &str,
    label: &Label,
    for_rendering: bool,
) {
    if node.uses_raster_paints && !for_rendering {
        warn!(
            "Potentially incorrect import result for resource {label}\n{}\n{}",
            "- The resource is being imported as a vector",
            format!("- The `{node_name}` node in Figma contains embedded raster images"),
        )
    }
    if node.uses_raster_paints && for_rendering {
        warn!(
            "Potentially incorrect import result for resource {label}\n{}\n{}\n{}\n{}",
            "- The resource is being imported as a vector",
            "- The resource is intended for vector-based rendering at multiple resolutions",
            format!("- The `{node_name}` node in Figma contains embedded raster images"),
            "If this behavior is intentional, consider enabling the `legacy_loader = true` option for this resource or for the entire profile."
        )
    }
}
