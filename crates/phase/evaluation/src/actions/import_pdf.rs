use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};
use crate::{EvalContext, Result, Target, figma::NodeMetadata};
use log::{debug, info};
use phase_loading::PdfProfile;

pub fn import_pdf(ctx: &EvalContext, args: ImportPdfArgs) -> Result<()> {
    let ImportPdfArgs {
        node,
        target,
        profile,
    } = args;
    let variant_name = target.id.clone().unwrap_or_default();

    debug!(target: "Import", "pdf: {}", target.attrs.label.name);
    let pdf = &get_remote_image(
        ctx,
        GetRemoteImageArgs {
            label: &target.attrs.label,
            remote: &target.attrs.remote,
            node,
            format: "pdf",
            scale: 1.0,
            variant_name: &variant_name,
        },
    )?;
    if ctx.eval_args.fetch {
        return Ok(());
    }

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
            file_extension: "pdf",
            bytes: pdf,
        },
        || info!(target: "Writing", "`{label}`{variant} to file"),
    )?;

    Ok(())
}

pub struct ImportPdfArgs<'a> {
    node: &'a NodeMetadata,
    target: Target<'a>,
    profile: &'a PdfProfile,
}

impl<'a> ImportPdfArgs<'a> {
    pub fn new(node: &'a NodeMetadata, target: Target<'a>, profile: &'a PdfProfile) -> Self {
        Self {
            node,
            target,
            profile,
        }
    }
}
