use super::materialize::{MaterializeArgs, materialize};
use crate::{
    EXPORTED_IMAGE_TAG, EvalContext, Result, Target,
    actions::{
        convert_png_to_webp::{ConvertPngToWebpArgs, convert_png_to_webp},
        render_svg_to_png::{RenderSvgToPngArgs, render_svg_to_png},
        validation::ensure_is_vector_node,
    },
    figma::NodeMetadata,
};
use lib_cache::CacheKey;
use log::{debug, info, warn};
use phase_loading::WebpProfile;

pub fn import_webp(ctx: &EvalContext, args: ImportWebpArgs) -> Result<()> {
    let ImportWebpArgs {
        node,
        target,
        profile,
    } = args;
    let node_name = target.figma_name();
    let scale = target.scale.unwrap_or(*profile.scale);
    let variant_name = target.id.clone().unwrap_or_default();

    debug!(target: "Import", "webp: {}", target.attrs.label.name);

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
    let webp = &convert_png_to_webp(
        ctx,
        ConvertPngToWebpArgs {
            quality: *args.profile.quality,
            bytes: &png,
            label: &target.attrs.label,
            variant_name: &variant_name,
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
            file_name: target.output_name(),
            file_extension: "webp",
            bytes: webp,
        },
        || info!(target: "Writing", "`{label}`{variant} to file"),
    )?;

    Ok(())
}

pub struct ImportWebpArgs<'a> {
    node: &'a NodeMetadata,
    target: Target<'a>,
    profile: &'a WebpProfile,
}

impl<'a> ImportWebpArgs<'a> {
    pub fn new(node: &'a NodeMetadata, target: Target<'a>, profile: &'a WebpProfile) -> Self {
        Self {
            node,
            target,
            profile,
        }
    }
}
