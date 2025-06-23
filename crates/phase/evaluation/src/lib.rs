use actions::{
    {ImportAndroidWebpArgs, import_android_webp}, {ImportComposeArgs, import_compose},
    {ImportPdfArgs, import_pdf}, {ImportPngArgs, import_png}, {ImportSvgArgs, import_svg},
    {ImportWebpArgs, import_webp},
};
use crossbeam_channel::unbounded;
use dashmap::DashMap;
use figma::FigmaRepository;
use lib_cache::{Cache, CacheConfig};
use lib_dashboard::{
    InitDashboardParams, init_dashboard, lifecycle, shutdown_dashboard, track_progress,
};
use lib_figma_fluent::FigmaApi;
use lib_metrics::{Counter, Metrics};
use log::{debug, error, trace};
use ordermap::OrderMap;
use phase_loading::{RemoteSource, Workspace};
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
    thread::available_parallelism,
    time::Duration,
};

pub mod actions;
mod error;
pub mod figma;
mod hashing;
// pub use actions_old::*;
pub use error::*;
pub use hashing::*;
mod targets;
pub use targets::*;

use crate::figma::{
    NodeMetadata,
    indexing::{RemoteIndex, Subscription, SubscriptionHandle},
};

#[derive(Clone)]
pub struct EvalContext {
    pub eval_args: Arc<EvalArgs>,
    pub figma_repository: FigmaRepository,
    pub cache: Cache,
    pub metrics: EvalMetrics,
}

#[derive(Clone)]
pub struct EvalMetrics {
    pub targets_evaluated: Arc<Counter>,
    pub targets_from_cache: Arc<Counter>,
}

#[derive(Default)]
pub struct EvalArgs {
    pub fetch: bool,
    pub refetch: bool,
    pub concurrency: usize,
    pub metrics: Metrics,
}

/// Maximum number of parallel jobs if user doesn't specify it explicitly
const MAX_NUM_THREADS: usize = 8;

pub fn evaluate(ws: Workspace, args: EvalArgs) -> Result<()> {
    let metrics = args.metrics.clone();
    let evaluation_duration = metrics.duration("figx_evaluation_duration");
    let _instant = evaluation_duration.record();
    // setup rayon thread pool
    set_up_rayon(args.concurrency);
    let ctx = init_eval_context(&ws, args, &metrics)?;
    let requested_remotes = ws
        .packages
        .iter()
        .flat_map(|pkg| &pkg.resources)
        .map(|res| &res.attrs.remote)
        .collect::<HashSet<_>>()
        .len();
    metrics
        .counter("figx_remotes_requested")
        .set(requested_remotes);

    // region: exec

    let mut remote_to_resources = OrderMap::<Arc<RemoteSource>, Vec<Target>>::new();
    let mut requested_targets = 0usize;
    let mut loaded_packages = 0usize;
    for pkg in ws.packages.iter() {
        loaded_packages += 1;
        for res in pkg.resources.iter() {
            let mut targets = targets_from_resource(res);
            requested_targets += targets.len();
            remote_to_resources
                .entry(res.attrs.remote.clone())
                .or_default()
                .append(&mut targets);
        }
    }
    metrics
        .counter("figx_targets_requested")
        .set(requested_targets);

    lifecycle!(
        target: "@Requested",
        "{tn} target{tp} from {rn} remote{rp} ({pn} package{pp} loaded)",
        tn = requested_targets,
        tp = if requested_targets == 1 { "" } else { "s" },
        rn = requested_remotes,
        rp = if requested_remotes == 1 { "" } else { "s" },
        pn = loaded_packages,
        pp = if loaded_packages == 1 { "" } else { "s" },
    );
    init_dashboard(InitDashboardParams {
        requested_targets,
        requested_remotes,
        loaded_packages,
        process_name: if ctx.eval_args.fetch {
            "Fetching"
        } else {
            "Importing"
        },
    });

    let result = remote_to_resources
        .into_iter()
        .par_bridge()
        .map(|(remote, targets)| {
            let index = RemoteIndex::new(FigmaApi::default(), ctx.cache.clone());
            let (handle, subscription) = index.subscribe(
                remote.as_ref(),
                ctx.eval_args.fetch || ctx.eval_args.refetch,
            )?;
            match subscription {
                Subscription::FromCache(name_to_node) => {
                    execute_with_cached_index(&ctx, targets, name_to_node)
                }
                Subscription::FromRemote(stream) => {
                    execute_with_streaming_index(&ctx, targets, stream, handle, remote.clone())
                }
            }
        })
        .collect::<Result<Vec<_>>>();

    // endregion: exec
    drop(_instant);
    shutdown_dashboard();

    // Извлекаем ошибку, если она была
    match result {
        Err(e) => Err(e),
        Ok(_) => {
            let time = format_duration(evaluation_duration.get());
            let targets_count = ctx.metrics.targets_evaluated.get();
            lifecycle!(
                target: "@Finished",
                "{targets_count} target{tp} in {time}",
                tp = if targets_count == 1 { "" } else { "s" },
            );
            Ok(())
        }
    }
}

fn execute_with_cached_index(
    ctx: &EvalContext,
    targets: Vec<Target>,
    name_to_node: HashMap<String, NodeMetadata>,
) -> Result<()> {
    targets.into_par_iter().try_for_each(|target| {
        let tracker = track_progress(target.attrs.label.name.to_string());
        let node = name_to_node
            .get(target.figma_name())
            .ok_or_else::<Error, _>(|| (&target).into())?;
        let result = import_target(target, ctx, &node);
        ctx.metrics.targets_evaluated.increment();
        tracker.mark_as_done();
        result
    })
}

fn execute_with_streaming_index(
    ctx: &EvalContext,
    targets: Vec<Target<'_>>,
    stream: Box<dyn Iterator<Item = Result<NodeMetadata>> + Send + '_>,
    handle: SubscriptionHandle,
    remote: Arc<RemoteSource>,
) -> Result<()> {
    // Group resources by their expected node name
    let name_to_targets: Arc<DashMap<_, Vec<_>>> = Arc::new(DashMap::with_capacity(targets.len()));
    for target in targets {
        name_to_targets
            .entry(target.figma_name().to_owned())
            .or_insert_with(|| Vec::with_capacity(1))
            .push(target);
    }

    let (tx, rx) = unbounded::<(Vec<Target>, NodeMetadata)>();
    let indexing_error: Arc<Mutex<Option<Error>>> = Default::default();
    let import_result = rayon::scope(|s| {
        let indexing_error = Arc::clone(&indexing_error);
        let name_to_targets = Arc::clone(&name_to_targets);
        s.spawn(move |_| {
            for node in stream {
                let node = match node {
                    Ok(node) => node,
                    Err(e) => {
                        *indexing_error.lock().unwrap() = Some(e);
                        return;
                    }
                };
                if let Some((_, targets)) = name_to_targets.remove(&node.name) {
                    let _ = tx.send((targets, node.clone()));
                }
            }
            if let Err(e) = handle.commit_cache() {
                error!("Unable to save indexed remote `{remote}` data to cache");
                *indexing_error.lock().unwrap() = Some(e)
            }
        });

        rx.iter().par_bridge().try_for_each(|(targets, node)| {
            // Bottleneck when multiple resources with the same figma_name appear
            // So we dedicate one thread entirely to process them sequentially
            // TODO: find a more efficient solution
            for target in targets {
                let tracker = track_progress(target.attrs.label.name.to_string());
                import_target(target, ctx, &node)?;
                ctx.metrics.targets_evaluated.increment();
                tracker.mark_as_done();
            }
            Ok(())
        })
    });

    // show NODE NOT FOUND error if needed
    if indexing_error.lock().unwrap().is_none() && import_result.is_ok() {
        for entry in name_to_targets.iter() {
            for res in entry.value() {
                return Err(res.into());
            }
        }
    }

    match (indexing_error.lock().unwrap().take(), import_result) {
        (Some(e), _) => Err(e),
        (_, res) => res,
    }
}

fn import_target(target: Target<'_>, ctx: &EvalContext, node: &NodeMetadata) -> Result<()> {
    use phase_loading::Profile::*;
    match target.profile {
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
    }
}

fn set_up_rayon(user_defined_concurrency: usize) {
    let num_threads = if user_defined_concurrency == 0 {
        let available = available_parallelism()
            .map(|it| it.get())
            .unwrap_or(MAX_NUM_THREADS);
        min(available, MAX_NUM_THREADS)
    } else {
        user_defined_concurrency
    };
    debug!(target: "Setup", "set rayon concurrency to {num_threads}");
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global();
}

pub fn setup_cache(dir: &Path) -> Result<Cache> {
    trace!("Ensuring all dirs to cache DB exists...");
    std::fs::create_dir_all(dir)?;
    debug!("Loading cache...");
    Ok(Cache::new(
        dir,
        CacheConfig {
            ignore_write_conflict: true,
            allow_deserialization_error: true,
        },
    )?)
}

fn init_eval_context(ws: &Workspace, args: EvalArgs, metrics: &Metrics) -> Result<EvalContext> {
    let api = FigmaApi::default();
    let cache = setup_cache(&ws.context.cache_dir)?;
    Ok(EvalContext {
        eval_args: Arc::new(args),
        figma_repository: FigmaRepository::new(api, cache.clone()),
        cache,
        metrics: EvalMetrics {
            targets_evaluated: metrics.counter("figx_targets_evaluated"),
            targets_from_cache: metrics.counter("figx_targets_from_cache"),
        },
    })
}

fn format_duration(duration: Duration) -> String {
    let total_millis = duration.as_millis();

    if total_millis < 1000 {
        return format!("{} sec", total_millis as f32 / 1000f32);
    }

    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;

    match (minutes, seconds) {
        (0, s) => format!("{} sec", s),
        (m, 0) => format!("{} min", m),
        (m, s) => format!("{} min {} sec", m, s),
    }
}
