use crate::figma::NodeMetadata;
use lib_label::Label;
use log::warn;

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
