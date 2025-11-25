use super::materialize::{MaterializeArgs, materialize};
use crate::{EXPORTED_IMAGE_TAG, EvalContext, Result, Target, figma::NodeMetadata};
use lib_cache::CacheKey;
use log::{debug, info, warn};
use phase_loading::PdfProfile;

pub fn import_pdf(ctx: &EvalContext, args: ImportPdfArgs) -> Result<()> {
    let ImportPdfArgs {
        node,
        target,
        profile,
    } = args;

    debug!(target: "Import", "pdf: {}", target.attrs.label.name);
    let image_cache_key = CacheKey::builder()
        .set_tag(EXPORTED_IMAGE_TAG)
        .write_str(&target.attrs.remote.file_key)
        .write_str(target.export_format())
        .write_str(&node.id)
        .write_u64(node.hash)
        .build();
    let Some(pdf) = ctx.cache.get_bytes(&image_cache_key)? else {
        warn!(target: "Importing", "internal: no image found by cache key");
        return Ok(());
    };
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
            bytes: &pdf,
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
