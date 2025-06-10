use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};
use crate::{
    EvalContext, Result,
    actions::{
        convert_png_to_webp::{ConvertPngToWebpArgs, convert_png_to_webp},
        util_variants::generate_variants,
    },
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
        args.profile.scale,
        &args.profile.variants,
    );

    variants
        .par_iter()
        .map(|variant| {
            let png = &get_remote_image(
                ctx,
                GetRemoteImageArgs {
                    label: &args.attrs.label,
                    remote: &args.attrs.remote,
                    node_name: &variant.node_name,
                    format: "png",
                    scale: variant.scale,
                },
            )?;
            let webp = &convert_png_to_webp(
                ctx,
                ConvertPngToWebpArgs {
                    quality: args.profile.quality,
                    bytes: png,
                    label: &args.attrs.label,
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
                || info!(target: "Writing", "`{}` to file", args.attrs.label.truncated_display(60)),
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
