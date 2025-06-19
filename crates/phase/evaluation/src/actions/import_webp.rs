use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};
use crate::{
    EvalContext, Result,
    actions::{
        convert_png_to_webp::{ConvertPngToWebpArgs, convert_png_to_webp},
        get_node::ensure_is_vector_node,
        render_svg_to_png::{RenderSvgToPngArgs, render_svg_to_png},
        util_variants::generate_variants,
    },
    figma::NodeMetadata,
};
use lib_progress_bar::create_in_progress_item;
use log::{debug, info};
use phase_loading::{ResourceAttrs, WebpProfile};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub fn import_webp(ctx: &EvalContext, args: ImportWebpArgs) -> Result<()> {
    debug!(target: "Import", "webp: {}", args.attrs.label.name);
    let _guard = create_in_progress_item(args.attrs.label.name.as_ref());

    let variants = generate_variants(
        &args.attrs.label.name.to_string(),
        &args.attrs.node_name,
        *args.profile.scale,
        &args.profile.variants,
    );

    variants
        .par_iter()
        .map(|variant| {
            let png = if args.profile.legacy_loader {
                get_remote_image(
                    ctx,
                    GetRemoteImageArgs {
                        label: &args.attrs.label,
                        remote: &args.attrs.remote,
                        node: &args.node,
                        format: "png",
                        scale: variant.scale,
                        variant_name: &variant.id,
                    },
                )?
            } else {
                ensure_is_vector_node(&args.node, &variant.node_name, &args.attrs.label, true);
                let svg = get_remote_image(
                    ctx,
                    GetRemoteImageArgs {
                        label: &args.attrs.label,
                        remote: &args.attrs.remote,
                        node: &args.node,
                        format: "svg",
                        scale: 1.0, // always the same yes
                        variant_name: "", // no variant yes
                    },
                )?;
                render_svg_to_png(
                    ctx,
                    RenderSvgToPngArgs {
                        label: &args.attrs.label,
                        variant_name: &variant.id,
                        svg: &svg,
                        zoom: if variant.scale != 1.0 { Some(variant.scale) } else { None },
                    },
                )?
            };
            let webp = &convert_png_to_webp(
                ctx,
                ConvertPngToWebpArgs {
                    quality: *args.profile.quality,
                    bytes: &png,
                    label: &args.attrs.label,
                    variant_name: &variant.id,
                },
            )?;
            materialize(
                ctx,
                MaterializeArgs {
                    output_dir: &args.attrs.package_dir.join(&args.profile.output_dir),
                    file_name: &variant.res_name,
                    file_extension: "webp",
                    bytes: webp,
                },
                || {
                    info!(target: "Writing", "`{label}`{variant} to file",
                        label = args.attrs.label.fitted(50),
                        variant = if variant.default { String::new() } else { format!(" ({})", variant.id) },
                    )
                },
            )
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

pub struct ImportWebpArgs<'a> {
    node: &'a NodeMetadata,
    attrs: &'a ResourceAttrs,
    profile: &'a WebpProfile,
}

impl<'a> ImportWebpArgs<'a> {
    pub fn new(node: &'a NodeMetadata, attrs: &'a ResourceAttrs, profile: &'a WebpProfile) -> Self {
        Self {
            node,
            attrs,
            profile,
        }
    }
}
