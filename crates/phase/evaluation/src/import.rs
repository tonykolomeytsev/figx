use crate::{
    Result,
    actions::{
        ImportAndroidDrawableArgs, ImportAndroidWebpArgs, ImportComposeArgs, ImportPdfArgs,
        ImportPngArgs, ImportSvgArgs, ImportWebpArgs, import_android_drawable, import_android_webp,
        import_compose, import_pdf, import_png, import_svg, import_webp,
    },
    figma::{NodeMetadata, indexing::RemoteIndex},
};
use image::EncodableLayout;
use lib_cache::CacheKey;
use lib_dashboard::track_progress;
use lib_figma_fluent::GetImageQueryParameters;
use log::{debug, info, warn};
use ordermap::OrderMap;
use phase_loading::RemoteSource;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{EvalContext, Target};

pub const REMOTE_SOURCE_TAG: u8 = 0x42;
pub const EXPORTED_IMAGE_TAG: u8 = 0x43;
pub const DOWNLOADED_IMAGE_TAG: u8 = 0x44;

pub fn import_all(ctx: EvalContext, r2t: OrderMap<Arc<RemoteSource>, Vec<Target>>) -> Result<()> {
    for (remote, targets) in r2t {
        // 0. Loading remote index to memory
        info!(target: "Importing", "loading remote index");
        let index = RemoteIndex::new(ctx.api.clone(), ctx.cache.clone())
            .load(&remote, ctx.eval_args.refetch || ctx.eval_args.fetch)?;

        // 1. Group targets by chunks
        // Chunks defined by NODE_NAME, EXPORT_FORMAT
        // EXPORT_SCALE is always 1.0, because we always export vector base and render it locally
        // grouped_targets: Map<ExportFormat, Vec<Target>>
        let mut grouped_targets = HashMap::new();
        for target in targets {
            grouped_targets
                .entry(target.export_format().to_owned())
                .or_insert(Vec::new())
                .push(target);
        }

        // 2. Run export against all grouped chunks
        for (export_format, targets) in grouped_targets {
            info!(target: "Importing", "batch: format={export_format}");
            import_chunk(&remote, &ctx, &index, &export_format, targets)?;
        }
    }
    Ok(())
}

fn import_chunk(
    remote: &RemoteSource,
    ctx: &EvalContext,
    index: &HashMap<String, NodeMetadata>,
    export_format: &str,
    targets: Vec<Target>,
) -> Result<()> {
    let chunk_cache_key_builder = CacheKey::builder()
        .set_tag(EXPORTED_IMAGE_TAG)
        .write_str(&remote.file_key)
        .write_str(export_format);

    // collect all node ids for export
    let mut ids_to_export = HashSet::with_capacity(targets.len());
    for target in &targets {
        let Some(node) = index.get(target.figma_name()) else {
            return Err(target.into());
        };
        // TODO: add only non-cached
        ids_to_export.insert(node.id.to_owned());
    }

    let ids_to_node = index
        .iter()
        .map(|(_, node)| (node.id.to_owned(), node))
        .collect::<HashMap<_, _>>();
    let ids_to_export = ids_to_export.into_iter().collect::<Vec<_>>();
    for sub_chunk in ids_to_export.chunks(500) {
        debug!(target: "Importing", "batch of size {} with format {export_format}", sub_chunk.len());
        let response = ctx.api.get_image(
            &remote.access_token,
            &remote.file_key,
            GetImageQueryParameters {
                ids: Some(sub_chunk),
                scale: Some(1.0),
                format: Some(export_format),
                ..Default::default()
            },
        )?;

        response
            .images
            .par_iter()
            .filter_map(|(node_id, link)| {
                if link.is_none() {
                    warn!(target: "Importing", "node with id '{node_id}' was not rendered");
                }
                link.as_ref().map(|link| (node_id, link))
            })
            .try_for_each::<_, crate::Result<()>>(|(node_id, link)| {
                let image_cache_key = chunk_cache_key_builder
                    .clone()
                    .write_str(&node_id)
                    .write_u64(
                        ids_to_node
                            .get(node_id)
                            .expect(&format!(
                                "node id {node_id} from response always present in index"
                            ))
                            .hash,
                    )
                    .build();
                let bytes = ctx.api.download_resource(&remote.access_token, &link)?;
                ctx.cache.put_bytes(&image_cache_key, &bytes.as_bytes())?;
                Ok(())
            })?;
    }

    targets.into_par_iter().try_for_each(|target| {
        let node = index.get(target.figma_name()).expect("already validated");
        import_target(target, &ctx, node)
    })?;

    Ok(())
}

fn import_target(target: Target<'_>, ctx: &EvalContext, node: &NodeMetadata) -> Result<()> {
    let _guard = track_progress(target.attrs.label.name.to_string());
    use phase_loading::Profile::*;
    let result = match target.profile {
        Png(png_profile) => import_png(&ctx, ImportPngArgs::new(node, target, png_profile)),
        Svg(svg_profile) => import_svg(&ctx, ImportSvgArgs::new(node, target, svg_profile)),
        Pdf(pdf_profile) => import_pdf(&ctx, ImportPdfArgs::new(node, target, pdf_profile)),
        Webp(webp_profile) => import_webp(&ctx, ImportWebpArgs::new(node, target, webp_profile)),
        Compose(compose_profile) => {
            import_compose(&ctx, ImportComposeArgs::new(node, target, compose_profile))
        }
        AndroidWebp(android_webp_profile) => import_android_webp(
            &ctx,
            ImportAndroidWebpArgs::new(node, target, android_webp_profile),
        ),
        AndroidDrawable(android_drawable_profile) => import_android_drawable(
            &ctx,
            ImportAndroidDrawableArgs::new(node, target, android_drawable_profile),
        ),
    };
    _guard.mark_as_done();
    result
}
