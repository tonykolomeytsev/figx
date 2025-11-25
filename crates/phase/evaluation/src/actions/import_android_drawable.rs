use crate::EXPORTED_IMAGE_TAG;
use crate::EvalContext;
use crate::Result;
use crate::Target;
use crate::actions::ConvertSvgToVectorDrawableArgs;
use crate::actions::convert_svg_to_vector_drawable;
use crate::actions::materialize::MaterializeArgs;
use crate::actions::materialize::materialize;
use crate::actions::validation::ensure_is_vector_node;
use crate::figma::NodeMetadata;
use lib_cache::CacheKey;
use log::debug;
use log::info;
use log::warn;
use phase_loading::AndroidDrawableProfile;

pub fn import_android_drawable(ctx: &EvalContext, args: ImportAndroidDrawableArgs) -> Result<()> {
    let ImportAndroidDrawableArgs {
        node,
        target,
        profile,
    } = args;
    let node_name = target.figma_name();
    let variant_name = target.id.clone().unwrap_or_default();

    debug!(target: "Import", "android-drawable: {}", target.attrs.label.name);
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

    let vector_drawable = convert_svg_to_vector_drawable(
        ctx,
        ConvertSvgToVectorDrawableArgs {
            svg: &svg,
            label: &target.attrs.label,
            variant_name: &variant_name,
        },
    )?;

    let drawable_dir_name = if variant_name.is_empty() {
        "drawable".to_string()
    } else {
        format!("drawable-{variant_name}")
    };
    let output_dir = target
        .attrs
        .package_dir
        .join(&profile.android_res_dir)
        .join(&drawable_dir_name);

    let variant = target
        .id
        .as_ref()
        .map(|it| format!(" ({it})"))
        .unwrap_or_default();
    let label = target.attrs.label.fitted(50);
    materialize(
        ctx,
        MaterializeArgs {
            output_dir: &output_dir,
            file_name: target.output_name(),
            file_extension: "xml",
            bytes: &vector_drawable,
        },
        || info!(target: "Writing", "`{label}`{variant} to file"),
    )?;

    Ok(())
}

pub struct ImportAndroidDrawableArgs<'a> {
    node: &'a NodeMetadata,
    target: Target<'a>,
    profile: &'a AndroidDrawableProfile,
}

impl<'a> ImportAndroidDrawableArgs<'a> {
    pub fn new(
        node: &'a NodeMetadata,
        target: Target<'a>,
        profile: &'a AndroidDrawableProfile,
    ) -> Self {
        Self {
            node,
            target,
            profile,
        }
    }
}
