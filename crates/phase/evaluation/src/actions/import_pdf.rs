use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};
use crate::{EvalContext, Result, actions::util_variants::generate_variants, figma::NodeMetadata};
use lib_progress_bar::create_in_progress_item;
use log::{debug, info};
use phase_loading::{PdfProfile, ResourceAttrs};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub fn import_pdf(ctx: &EvalContext, args: ImportPdfArgs) -> Result<()> {
    debug!(target: "Import", "pdf: {}", args.attrs.label.name);
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
            let pdf = &get_remote_image(
                ctx,
                GetRemoteImageArgs {
                    label: &args.attrs.label,
                    remote: &args.attrs.remote,
                    node: &args.node,
                    format: "pdf",
                    scale: variant.scale,
                    variant_name: &variant.id,
                },
            )?;
            if ctx.eval_args.fetch {
                return Ok(());
            }
            materialize(
                ctx,
                MaterializeArgs {
                    output_dir: &args.attrs.package_dir.join(&args.profile.output_dir),
                    file_name: &variant.res_name,
                    file_extension: "pdf",
                    bytes: pdf,
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

pub struct ImportPdfArgs<'a> {
    node: &'a NodeMetadata,
    attrs: &'a ResourceAttrs,
    profile: &'a PdfProfile,
}

impl<'a> ImportPdfArgs<'a> {
    pub fn new(node: &'a NodeMetadata, attrs: &'a ResourceAttrs, profile: &'a PdfProfile) -> Self {
        Self {
            node,
            attrs,
            profile,
        }
    }
}
