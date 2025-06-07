use lib_progress_bar::create_in_progress_item;
use log::{debug, info};
use phase_loading::{PngProfile, ResourceAttrs};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{EvalContext, Result, actions::util_variants::generate_variants};

use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};

pub fn import_png(ctx: &EvalContext, args: ImportPngArgs) -> Result<()> {
    debug!(target: "Import", "png: {}", args.attrs.label.name);
    let _guard = create_in_progress_item(args.attrs.label.name.as_ref());

    let variants = generate_variants(
        &args.attrs.label.name.to_string(),
        &args.attrs.node_name,
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
                    scale: args.profile.scale,
                },
            )?;
            materialize(
                ctx,
                MaterializeArgs {
                    output_dir: &args.attrs.package_dir.join(&args.profile.output_dir),
                    file_name: &variant.res_name,
                    file_extension: "png",
                    bytes: png,
                },
                || info!(target: "Writing", "`{}` to file", args.attrs.label.truncated_display(60)),
            )
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

pub struct ImportPngArgs<'a> {
    attrs: &'a ResourceAttrs,
    profile: &'a PngProfile,
}

impl<'a> ImportPngArgs<'a> {
    pub fn new(attrs: &'a ResourceAttrs, profile: &'a PngProfile) -> Self {
        Self { attrs, profile }
    }
}
