use actions::{
    import_android_webp::{ImportAndroidWebpArgs, import_android_webp},
    import_compose::{ImportComposeArgs, import_compose},
    import_pdf::{ImportPdfArgs, import_pdf},
    import_png::{ImportPngArgs, import_png},
    import_svg::{ImportSvgArgs, import_svg},
    import_webp::{ImportWebpArgs, import_webp},
};
use crossbeam_channel::unbounded;
use dashmap::DashMap;
use figma::FigmaRepository;
use lib_cache::{Cache, CacheConfig};
use lib_figma_fluent::FigmaApi;
use lib_progress_bar::{
    create_in_progress_item, set_progress_bar_maximum, set_progress_bar_progress,
    set_progress_bar_visible,
};
use log::{debug, error, info, trace};
use ordermap::OrderMap;
use phase_loading::{RemoteSource, Resource, Workspace};
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

pub mod actions;
mod error;
pub mod figma;
mod hashing;
// pub use actions_old::*;
pub use error::*;
pub use hashing::*;

use crate::figma::{
    NodeMetadata,
    indexing::{RemoteIndex, Subscription, SubscriptionHandle},
};

#[derive(Clone)]
pub struct EvalContext {
    pub eval_args: Arc<EvalArgs>,
    pub api: FigmaApi,
    pub figma_repository: FigmaRepository,
    pub cache: Cache,
    pub actual_concurrency: usize,
    pub metrics: Arc<EvalMetrics>,
}

#[derive(Default)]
pub struct EvalMetrics {
    pub files_counter: AtomicUsize,
    pub res_counter: AtomicUsize,
}

#[derive(Default, Clone, Copy)]
pub struct EvalArgs {
    pub fetch: bool,
    pub refetch: bool,
    pub concurrency: usize,
}

/// Maximum number of parallel jobs if user doesn't specify it explicitly
const MAX_NUM_THREADS: usize = 8;

pub fn evaluate(ws: Workspace, args: EvalArgs) -> Result<()> {
    let instant = Instant::now();
    // setup rayon thread pool
    let actual_concurrency = set_up_rayon(args.concurrency);
    let ctx = init_eval_context(&ws, args, actual_concurrency)?;
    set_progress_bar_visible(true);
    let requested_resources = ws.packages.iter().map(|pkg| pkg.resources.len()).sum();
    let requested_remotes = ws
        .packages
        .iter()
        .flat_map(|pkg| &pkg.resources)
        .map(|res| &res.attrs.remote)
        .collect::<HashSet<_>>()
        .len();
    set_progress_bar_maximum(requested_resources);
    if ctx.eval_args.fetch {
        info!(target: "Requested", "update for {requested_remotes} remote(s)");
    } else {
        info!(target: "Requested", "{requested_resources} resource(s) from {requested_remotes} remote(s)");
    }

    // region: exec

    let mut remote_to_resources = OrderMap::<Arc<RemoteSource>, Vec<Resource>>::new();
    for pkg in ws.packages {
        for res in pkg.resources {
            remote_to_resources
                .entry(res.attrs.remote.clone())
                .or_default()
                .push(res);
        }
    }

    let result = remote_to_resources
        .into_iter()
        .par_bridge()
        .map(|(remote, resources)| {
            let index = RemoteIndex::new(FigmaApi::default(), ctx.cache.clone());
            let (handle, subscription) = index.subscribe(remote.as_ref(), args.refetch)?;
            match subscription {
                Subscription::FromCache(name_to_node) => {
                    execute_with_cached_index(&ctx, resources, name_to_node)
                }
                Subscription::FromRemote(stream) => {
                    execute_with_streaming_index(&ctx, resources, stream, handle, remote.clone())
                }
            }
        })
        .collect::<Result<Vec<_>>>();

    // endregion: exec
    let elapsed = instant.elapsed();
    set_progress_bar_visible(false);

    // Извлекаем ошибку, если она была
    match result {
        Err(e) => Err(e),
        Ok(_) => {
            let time = format_duration(elapsed);
            if ctx.eval_args.fetch {
                info!(target: "Finished", "{requested_remotes} remotes(s) in {time}",);
            } else {
                let files_count = ctx.metrics.files_counter.load(Ordering::Relaxed);
                info!(
                    target: "Finished", "{res_num} resource(s), resulting in {files_count} file(s) in {time}",
                    res_num = ctx.metrics.res_counter.load(Ordering::Relaxed),
                );
            }
            Ok(())
        }
    }
}

fn execute_with_streaming_index(
    ctx: &EvalContext,
    resources: Vec<Resource>,
    stream: Box<dyn Iterator<Item = Result<NodeMetadata>> + Send + '_>,
    handle: SubscriptionHandle,
    remote: Arc<RemoteSource>,
) -> Result<()> {
    // Group resources by their expected node name
    let name_to_resources: Arc<DashMap<_, Vec<_>>> =
        Arc::new(DashMap::with_capacity(resources.len()));
    for res in resources {
        name_to_resources
            .entry(res.attrs.node_name.clone())
            .or_insert_with(|| Vec::with_capacity(1))
            .push(res);
    }

    let (tx, rx) = unbounded::<(Resource, NodeMetadata)>();
    let indexing_error: Arc<Mutex<Option<Error>>> = Default::default();
    let import_result = rayon::scope(|s| {
        let indexing_error = Arc::clone(&indexing_error);
        let name_to_resources = Arc::clone(&name_to_resources);
        s.spawn(move |_| {
            let _guard = create_in_progress_item("REMOTE");
            for node in stream {
                let node = match node {
                    Ok(node) => node,
                    Err(e) => {
                        *indexing_error.lock().unwrap() = Some(e);
                        return;
                    }
                };
                if let Some((_, resources)) = name_to_resources.remove(&node.name) {
                    for res in resources {
                        let node = node.clone();
                        let _ = tx.send((res, node));
                    }
                }
            }
            if let Err(e) = handle.commit_cache() {
                error!("Unable to save indexed remote `{remote}` data to cache");
                *indexing_error.lock().unwrap() = Some(e)
            }
        });

        rx.iter().par_bridge().try_for_each(|(res, node)| {
            import_resource(&res, ctx, &node)?;
            ctx.metrics.res_counter.fetch_add(1, Ordering::SeqCst);
            set_progress_bar_progress(ctx.metrics.res_counter.load(Ordering::SeqCst));
            Ok(())
        })
    });

    // show NODE NOT FOUND error if needed
    if indexing_error.lock().unwrap().is_none() && import_result.is_ok() {
        for entry in name_to_resources.iter() {
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

fn execute_with_cached_index(
    ctx: &EvalContext,
    resources: Vec<Resource>,
    name_to_node: HashMap<String, NodeMetadata>,
) -> Result<()> {
    resources.into_par_iter().try_for_each(|res| {
        let node = name_to_node
            .get(&res.attrs.node_name)
            .ok_or_else::<Error, _>(|| (&res).into())?;
        let result = import_resource(&res, ctx, &node);
        ctx.metrics.res_counter.fetch_add(1, Ordering::SeqCst);
        set_progress_bar_progress(ctx.metrics.res_counter.load(Ordering::SeqCst));
        result
    })
}

fn import_resource(res: &Resource, ctx: &EvalContext, node: &NodeMetadata) -> Result<()> {
    use phase_loading::Profile::*;
    match res.profile.as_ref() {
        Png(png_profile) => import_png(&ctx, ImportPngArgs::new(node, &res.attrs, png_profile)),
        Svg(svg_profile) => import_svg(&ctx, ImportSvgArgs::new(node, &res.attrs, svg_profile)),
        Pdf(pdf_profile) => import_pdf(&ctx, ImportPdfArgs::new(node, &res.attrs, pdf_profile)),
        Webp(webp_profile) => {
            import_webp(&ctx, ImportWebpArgs::new(node, &res.attrs, webp_profile))
        }
        Compose(compose_profile) => import_compose(
            &ctx,
            ImportComposeArgs::new(node, &res.attrs, compose_profile),
        ),
        AndroidWebp(android_webp_profile) => import_android_webp(
            &ctx,
            ImportAndroidWebpArgs::new(node, &res.attrs, android_webp_profile),
        ),
    }
}

fn set_up_rayon(user_defined_concurrency: usize) -> usize {
    let num_threads = if user_defined_concurrency == 0 {
        min(num_cpus::get(), MAX_NUM_THREADS)
    } else {
        user_defined_concurrency
    };
    debug!(target: "Setup", "set rayon concurrency to {num_threads}");
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global();
    num_threads
}

pub fn setup_cache(dir: &Path) -> Result<Cache> {
    trace!("Ensuring all dirs to cache DB exists...");
    std::fs::create_dir_all(dir)?;
    debug!("Loading cache...");
    Ok(Cache::new(
        dir,
        CacheConfig {
            allow_deserialization_error: true,
            ignore_write_conflict: true,
        },
    )?)
}

fn init_eval_context(
    ws: &Workspace,
    args: EvalArgs,
    actual_concurrency: usize,
) -> Result<EvalContext> {
    let api = FigmaApi::default();
    let cache = setup_cache(&ws.context.cache_dir)?;
    Ok(EvalContext {
        eval_args: Arc::new(args),
        api: api.clone(),
        figma_repository: FigmaRepository::new(api, cache.clone()),
        cache,
        actual_concurrency,
        metrics: Default::default(),
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
