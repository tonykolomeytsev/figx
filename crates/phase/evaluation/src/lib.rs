use actions::{
    import_android_webp::{ImportAndroidWebpArgs, import_android_webp},
    import_compose::{ImportComposeArgs, import_compose},
    import_pdf::{ImportPdfArgs, import_pdf},
    import_png::{ImportPngArgs, import_png},
    import_svg::{ImportSvgArgs, import_svg},
    import_webp::{ImportWebpArgs, import_webp},
};
use figma::FigmaRepository;
use lib_cache::Cache;
use lib_figma::FigmaApi;
use lib_progress_bar::{set_progress_bar_maximum, set_progress_bar_visible};
use log::{debug, info, trace};
use phase_loading::Workspace;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    cmp::min,
    collections::HashSet,
    path::Path,
    sync::{
        Arc,
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

#[derive(Clone)]
pub struct EvalContext {
    pub eval_args: Arc<EvalArgs>,
    pub figma_repository: FigmaRepository,
    pub cache: Cache,
}

pub struct EvalArgs {
    pub refetch: bool,
    pub diagnostics: bool,
}

impl Default for EvalArgs {
    fn default() -> Self {
        Self {
            refetch: false,
            diagnostics: false,
        }
    }
}

const MAX_NUM_THREADS: usize = 6;

pub fn evaluate(ws: Workspace, args: EvalArgs) -> Result<()> {
    let instant = Instant::now();
    let ctx = init_eval_context(&ws, args)?;
    // setup rayon thread pool
    fun_name();
    set_progress_bar_visible(true);
    let requested_resources = ws.packages.iter().map(|pkg| pkg.resources.len()).sum();
    let processed_resources: Arc<AtomicUsize> = Default::default();
    let requested_remotes = ws
        .packages
        .iter()
        .flat_map(|pkg| &pkg.resources)
        .map(|res| &res.attrs.remote)
        .collect::<HashSet<_>>()
        .len();
    set_progress_bar_maximum(requested_resources);
    info!(target: "Requested", "{requested_resources} resource(s) from {requested_remotes} remote(s)");

    // region: exec
    let result = ws
        .packages
        .par_iter()
        .flat_map(|it| &it.resources)
        .map(|res| {
            use phase_loading::Profile::*;
            let result = match res.profile.as_ref() {
                Png(png_profile) => import_png(&ctx, ImportPngArgs::new(&res.attrs, &png_profile)),
                Svg(svg_profile) => import_svg(&ctx, ImportSvgArgs::new(&res.attrs, &svg_profile)),
                Pdf(pdf_profile) => import_pdf(&ctx, ImportPdfArgs::new(&res.attrs, pdf_profile)),
                Webp(webp_profile) => {
                    import_webp(&ctx, ImportWebpArgs::new(&res.attrs, webp_profile))
                }
                Compose(compose_profile) => {
                    import_compose(&ctx, ImportComposeArgs::new(&res.attrs, compose_profile))
                }
                AndroidWebp(android_webp_profile) => import_android_webp(
                    &ctx,
                    ImportAndroidWebpArgs::new(&res.attrs, android_webp_profile),
                ),
            };
            processed_resources.fetch_add(1, Ordering::Relaxed);
            lib_progress_bar::set_progress_bar_progress(
                processed_resources.load(Ordering::Relaxed),
            );
            result
        })
        .collect::<Result<()>>();
    // endregion: exec
    let elapsed = instant.elapsed();
    set_progress_bar_visible(false);

    // Извлекаем ошибку, если она была
    match result {
        Err(e) => Err(e),
        Ok(_) => {
            info!(
                target: "Finished", "{res_num} resource(s) in {time}",
                res_num = processed_resources.load(Ordering::Relaxed),
                time = format_duration(elapsed),
            );
            Ok(())
        }
    }
}

fn fun_name() {
    let num_threads = min(num_cpus::get(), MAX_NUM_THREADS);
    debug!(target: "Setup", "set rayon concurrency to {num_threads}");
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global();
}

pub fn setup_cache(dir: &Path) -> Result<Cache> {
    trace!("Ensuring all dirs to cache DB exists...");
    std::fs::create_dir_all(dir)?;
    debug!("Loading cache...");
    Ok(Cache::new(dir)?)
}

fn init_eval_context(ws: &Workspace, args: EvalArgs) -> Result<EvalContext> {
    let api = FigmaApi::default();
    let cache = setup_cache(&ws.context.cache_dir)?;
    Ok(EvalContext {
        eval_args: Arc::new(args),
        figma_repository: FigmaRepository::new(api, cache.clone()),
        cache,
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
