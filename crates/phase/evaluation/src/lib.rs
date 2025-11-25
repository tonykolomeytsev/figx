use lib_cache::{Cache, CacheConfig};
use lib_dashboard::{InitDashboardParams, init_dashboard, lifecycle, shutdown_dashboard};
use lib_figma_fluent::FigmaApi;
use lib_metrics::{Counter, Metrics};
use log::{debug, trace};
use ordermap::OrderMap;
use phase_loading::{RemoteSource, Workspace};
use std::{
    cmp::min, collections::HashSet, path::Path, sync::Arc, thread::available_parallelism,
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
mod import;
pub use import::*;

#[derive(Clone)]
pub struct EvalContext {
    pub eval_args: Arc<EvalArgs>,
    pub api: FigmaApi,
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

    let mut remote_to_targets = OrderMap::<Arc<RemoteSource>, Vec<Target>>::new();
    let mut requested_targets = 0usize;
    let mut loaded_packages = 0usize;
    for pkg in ws.packages.iter() {
        loaded_packages += 1;
        for res in pkg.resources.iter() {
            let mut targets = targets_from_resource(res);
            requested_targets += targets.len();
            remote_to_targets
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
        process_name: if args.fetch { "Fetching" } else { "Importing" },
    });

    let ctx = init_eval_context(&ws, args, &metrics)?;

    let result = import_all(ctx.clone(), remote_to_targets);

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
    let cache = setup_cache(&ws.context.cache_dir)?;
    Ok(EvalContext {
        eval_args: Arc::new(args),
        api: FigmaApi::default(),
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
