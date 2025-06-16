use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};
use crate::{
    actions::{
        convert_png_to_webp::{convert_png_to_webp, ConvertPngToWebpArgs}, get_node::{ensure_is_vector_node, get_node, GetNodeArgs}, render_svg_to_png::{render_svg_to_png, RenderSvgToPngArgs}, util_variants::generate_variants
    }, EvalContext, Result
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
            let node = get_node(ctx, GetNodeArgs { 
                node_name: &variant.node_name, 
                remote: &args.attrs.remote 
            })?;
            let png = if args.profile.legacy_loader {
                get_remote_image(
                    ctx,
                    GetRemoteImageArgs {
                        label: &args.attrs.label,
                        remote: &args.attrs.remote,
                        node: &node,
                        format: "png",
                        scale: variant.scale,
                        variant_name: &variant.id,
                    },
                )?
            } else {
                ensure_is_vector_node(&node, &variant.node_name, &args.attrs.label, true);
                let svg = get_remote_image(
                    ctx,
                    GetRemoteImageArgs {
                        label: &args.attrs.label,
                        remote: &args.attrs.remote,
                        node: &node,
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
    attrs: &'a ResourceAttrs,
    profile: &'a WebpProfile,
}

impl<'a> ImportWebpArgs<'a> {
    pub fn new(attrs: &'a ResourceAttrs, profile: &'a WebpProfile) -> Self {
        Self { attrs, profile }
    }
}
