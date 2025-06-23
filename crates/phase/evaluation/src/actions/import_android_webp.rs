use crate::EvalContext;
use crate::Result;
use crate::Target;
use crate::actions::GetRemoteImageArgs;
use crate::actions::convert_png_to_webp::ConvertPngToWebpArgs;
use crate::actions::convert_png_to_webp::convert_png_to_webp;
use crate::actions::get_remote_image;
use crate::actions::materialize::MaterializeArgs;
use crate::actions::materialize::materialize;
use crate::actions::render_svg_to_png::RenderSvgToPngArgs;
use crate::actions::render_svg_to_png::render_svg_to_png;
use crate::actions::validation::ensure_is_vector_node;
use crate::figma::NodeMetadata;
use log::debug;
use log::info;
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
                node,
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
                variant_name: &variant_name,
                svg: &svg,
                zoom: if scale != 1.0 { Some(scale) } else { None },
            },
        )?
    };
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
