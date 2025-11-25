use crate::{
    EXPORTED_IMAGE_TAG, EvalContext, Result, Target,
    actions::{
        render_svg_to_png::{RenderSvgToPngArgs, render_svg_to_png},
        validation::ensure_is_vector_node,
    },
    figma::NodeMetadata,
};
use lib_cache::CacheKey;
use log::{debug, info, warn};
use phase_loading::PngProfile;

use super::materialize::{MaterializeArgs, materialize};

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

    ensure_is_vector_node(&node, node_name, &target.attrs.label, true);
    let image_cache_key = CacheKey::builder()
        .set_tag(EXPORTED_IMAGE_TAG)
        .write_str(&target.attrs.remote.file_key)
        .write_str(target.export_format())
        .write_str(&node.id)
        .write_u64(node.hash)
        .build();
    let Some(svg) = ctx.cache.get_bytes(&image_cache_key)? else {
        warn!(target: "Importing", "internal: no image found by cache key");
        return Ok(());
    };
    if ctx.eval_args.fetch {
        return Ok(());
    }
    let png = render_svg_to_png(
        ctx,
        RenderSvgToPngArgs {
            label: &target.attrs.label,
            variant_name: &variant_name,
            svg: &svg,
            zoom: if scale != 1.0 { Some(scale) } else { None },
        },
    )?;

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
