use crate::{
    EvalContext, Result, Target,
    actions::{
        get_node::{GetNodeArgs, ensure_is_vector_node, get_node},
        render_svg_to_png::{RenderSvgToPngArgs, render_svg_to_png},
    },
};
use lib_progress_bar::create_in_progress_item;
use log::{debug, info};
use phase_loading::PngProfile;

use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};

pub fn import_png(ctx: &EvalContext, args: ImportPngArgs) -> Result<()> {
    let ImportPngArgs { target, profile } = args;
    let node_name = target.figma_name();
    let scale = target.scale.unwrap_or(*profile.scale);
    let variant_name = target.id.clone().unwrap_or_default();

    debug!(target: "Import", "png: {}", target.attrs.label.name);
    let _guard = create_in_progress_item(target.attrs.label.name.as_ref());

    let node = get_node(
        ctx,
        GetNodeArgs {
            node_name,
            remote: &target.attrs.remote,
            diag: &target.attrs.diag,
        },
    )?;
    let png = if profile.legacy_loader {
        get_remote_image(
            ctx,
            GetRemoteImageArgs {
                label: &target.attrs.label,
                remote: &target.attrs.remote,
                node: &node,
                format: "png",
                scale,
                variant_name: &variant_name,
            },
        )?
    } else {
        ensure_is_vector_node(&node, node_name, &target.attrs.label, true);
        let svg = get_remote_image(
            ctx,
            GetRemoteImageArgs {
                label: &target.attrs.label,
                remote: &target.attrs.remote,
                node: &node,
                format: "svg",
                scale: 1.0,       // always the same yes
                variant_name: "", // no variant yes
            },
        )?;
        render_svg_to_png(
            ctx,
            RenderSvgToPngArgs {
                label: &target.attrs.label,
                variant_name: &target.id.clone().unwrap_or_default(),
                svg: &svg,
                zoom: if scale != 1.0 { Some(scale) } else { None },
            },
        )?
    };

    let variant = target
        .id
        .as_ref()
        .map(|it| format!(" ({it})"))
        .unwrap_or_default();
    let label = target.attrs.label.fitted(50);
    materialize(
        ctx,
        MaterializeArgs {
            output_dir: &target.attrs.package_dir.join(&profile.output_dir),
            file_name: &target.output_name(),
            file_extension: "png",
            bytes: &png,
        },
        || info!(target: "Writing", "`{label}`{variant} to file"),
    )?;

    Ok(())
}

pub struct ImportPngArgs<'a> {
    target: Target<'a>,
    profile: &'a PngProfile,
}

impl<'a> ImportPngArgs<'a> {
    pub fn new(target: Target<'a>, profile: &'a PngProfile) -> Self {
        Self { target, profile }
    }
}
