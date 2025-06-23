use crate::{
    EvalContext, Result, Target,
    actions::{
        render_svg_to_png::{RenderSvgToPngArgs, render_svg_to_png},
        validation::ensure_is_vector_node,
    },
    figma::NodeMetadata,
};
use log::{debug, info};
use phase_loading::PngProfile;

use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};

pub fn import_png(ctx: &EvalContext, args: ImportPngArgs) -> Result<()> {
    let ImportPngArgs {
        node,
        target,
        profile,
    } = args;
    let node_name = target.figma_name();
    let scale = target.scale.unwrap_or(*profile.scale);
    let variant_name = target.id.clone().unwrap_or_default();

    debug!(target: "Import", "png: {}", target.attrs.label.name);
    let png = if profile.legacy_loader {
        let png = get_remote_image(
            ctx,
            GetRemoteImageArgs {
                label: &target.attrs.label,
                remote: &target.attrs.remote,
                node,
                format: "png",
                scale,
                variant_name: &variant_name,
            },
        )?;
        if ctx.eval_args.fetch {
            return Ok(());
        }
        png
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
        if ctx.eval_args.fetch {
            return Ok(());
        }
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
    node: &'a NodeMetadata,
    target: Target<'a>,
    profile: &'a PngProfile,
}

impl<'a> ImportPngArgs<'a> {
    pub fn new(node: &'a NodeMetadata, target: Target<'a>, profile: &'a PngProfile) -> Self {
        Self {
            node,
            target,
            profile,
        }
    }
}
