use lib_progress_bar::create_in_progress_item;
use log::{debug, info};
use phase_loading::{ResourceAttrs, SvgProfile};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{EvalContext, Result, actions::util_variants::generate_variants};

use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};

pub fn import_svg(ctx: &EvalContext, args: ImportSvgArgs) -> Result<()> {
    debug!(target: "Import", "svg: {}", args.attrs.label.name);
    let _guard = create_in_progress_item(args.attrs.label.name.as_ref());

    let variants = generate_variants(
        &args.attrs.label.name.to_string(),
        &args.attrs.node_name,
        1.0,
        &args.profile.variants,
    );

    variants
        .par_iter()
        .map(|variant| {
            let svg = get_remote_image(
                ctx,
                GetRemoteImageArgs {
                    label: &args.attrs.label,
                    remote: &args.attrs.remote,
                    node_name: &variant.node_name,
                    format: "svg",
                    scale: variant.scale,
                    variant_name: &variant.id,
                },
            )?;
            materialize(
                ctx,
                MaterializeArgs {
                    output_dir: &args.attrs.package_dir.join(&args.profile.output_dir),
                    file_name: &variant.res_name,
                    file_extension: "svg",
                    bytes: &svg,
                },
                || {
                    info!(target: "Writing", "`{label}`{variant} to file",
                        label = args.attrs.label.fitted(50),
                        variant = if variant.default { String::new() } else { format!(" ({})", variant.id) },
                    )
                },            )
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

pub struct ImportSvgArgs<'a> {
    attrs: &'a ResourceAttrs,
    profile: &'a SvgProfile,
}

impl<'a> ImportSvgArgs<'a> {
    pub fn new(attrs: &'a ResourceAttrs, profile: &'a SvgProfile) -> Self {
        Self { attrs, profile }
    }
}
