use crate::EXPORTED_IMAGE_TAG;
use crate::EvalContext;
use crate::Result;
use crate::Target;
use crate::actions::convert_png_to_webp::ConvertPngToWebpArgs;
use crate::actions::convert_png_to_webp::convert_png_to_webp;
use crate::actions::materialize::MaterializeArgs;
use crate::actions::materialize::materialize;
use crate::actions::render_svg_to_png::RenderSvgToPngArgs;
use crate::actions::render_svg_to_png::render_svg_to_png;
use crate::actions::validation::ensure_is_vector_node;
use crate::figma::NodeMetadata;
use lib_cache::CacheKey;
use log::debug;
use log::info;
use log::warn;
use phase_loading::AndroidWebpProfile;

pub fn import_android_webp(ctx: &EvalContext, args: ImportAndroidWebpArgs) -> Result<()> {
    let ImportAndroidWebpArgs {
        node,
        target,
        profile,
    } = args;
    let node_name = target.figma_name();
    let scale = target.scale.expect("always present");
    let variant_name = target.id.clone().unwrap_or_default();

    debug!(target: "Import", "android-webp: {}", target.attrs.label.name);

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

    let webp = convert_png_to_webp(
        ctx,
        ConvertPngToWebpArgs {
            quality: *profile.quality,
            bytes: &png,
            label: &target.attrs.label,
            variant_name: &variant_name,
        },
    )?;
    let output_dir = target
        .attrs
        .package_dir
        .join(&profile.android_res_dir)
        .join(&format!("drawable-{variant_name}"));

    let variant = &variant_name;
    let label = target.attrs.label.fitted(50);
    materialize(
        ctx,
        MaterializeArgs {
            output_dir: &output_dir,
            file_name: target.attrs.label.name.as_ref(), // always the same name
            file_extension: "webp",
            bytes: &webp,
        },
        || info!(target: "Writing", "`{label}` ({variant}) to file"),
    )?;
    Ok(())
}

pub struct ImportAndroidWebpArgs<'a> {
    node: &'a NodeMetadata,
    target: Target<'a>,
    profile: &'a AndroidWebpProfile,
}

impl<'a> ImportAndroidWebpArgs<'a> {
    pub fn new(
        node: &'a NodeMetadata,
        target: Target<'a>,
        profile: &'a AndroidWebpProfile,
    ) -> Self {
        Self {
            node,
            target,
            profile,
        }
    }
}
