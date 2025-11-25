use crate::{
    EXPORTED_IMAGE_TAG, EvalContext, Result, Target,
    actions::{
        materialize::{MaterializeArgs, materialize},
        validation::ensure_is_vector_node,
    },
    figma::NodeMetadata,
};
use lib_cache::CacheKey;
use log::{debug, info, warn};
use phase_loading::SvgProfile;

pub fn import_svg(ctx: &EvalContext, args: ImportSvgArgs) -> Result<()> {
    let ImportSvgArgs {
        node,
        target,
        profile,
    } = args;
    let node_name = target.figma_name();

    debug!(target: "Import", "svg: {}", target.attrs.label.name);
    ensure_is_vector_node(&node, node_name, &target.attrs.label, false);
    let image_cache_key = CacheKey::builder()
        .set_tag(EXPORTED_IMAGE_TAG)
        .write_str(&target.attrs.remote.file_key)
        .write_str(target.export_format())
        .write_str(&node.id)
        .write_u64(node.hash)
        .build();
    let Some(svg) = ctx.cache.get_bytes(&image_cache_key)? else {
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
            file_extension: "svg",
            bytes: &svg,
        },
        || info!(target: "Writing", "`{label}`{variant} to file"),
    )?;

    Ok(())
}

pub struct ImportSvgArgs<'a> {
    node: &'a NodeMetadata,
    target: Target<'a>,
    profile: &'a SvgProfile,
}

impl<'a> ImportSvgArgs<'a> {
    pub fn new(node: &'a NodeMetadata, target: Target<'a>, profile: &'a SvgProfile) -> Self {
        Self {
            node,
            target,
            profile,
        }
    }
}
