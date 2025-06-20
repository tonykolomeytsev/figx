use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};
use crate::{
    EvalContext, Result, Target,
    actions::get_node::{GetNodeArgs, get_node},
};
use lib_progress_bar::create_in_progress_item;
use log::{debug, info};
use phase_loading::PdfProfile;

pub fn import_pdf(ctx: &EvalContext, args: ImportPdfArgs) -> Result<()> {
    let ImportPdfArgs { target, profile } = args;
    let node_name = target.figma_name();
    let variant_name = target.id.clone().unwrap_or_default();

    debug!(target: "Import", "pdf: {}", target.attrs.label.name);
    let _guard = create_in_progress_item(target.attrs.label.name.as_ref());

    let node = get_node(
        ctx,
        GetNodeArgs {
            node_name,
            remote: &target.attrs.remote,
            diag: &target.attrs.diag,
        },
    )?;
    let pdf = &get_remote_image(
        ctx,
        GetRemoteImageArgs {
            label: &target.attrs.label,
            remote: &target.attrs.remote,
            node: &node,
            format: "pdf",
            scale: 1.0,
            variant_name: &variant_name,
        },
    )?;

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
    target: Target<'a>,
    profile: &'a PdfProfile,
}

impl<'a> ImportPdfArgs<'a> {
    pub fn new(target: Target<'a>, profile: &'a PdfProfile) -> Self {
        Self { target, profile }
    }
}
