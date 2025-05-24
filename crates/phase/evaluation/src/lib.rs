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
use log::{debug, info, trace};
use phase_loading::Workspace;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    path::Path,
    sync::Arc,
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

const DEFAULT_NUM_THREADS: usize = 5;

pub fn evaluate(ws: Workspace, args: EvalArgs) -> Result<()> {
    let instant = Instant::now();
    let ctx = init_eval_context(&ws, args)?;
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(DEFAULT_NUM_THREADS)
        .build_global();

    // region: exec
    let result = ws
        .packages
        .par_iter()
        .flat_map(|it| &it.resources)
        .map(|res| {
            use phase_loading::Profile::*;
            match res.profile.as_ref() {
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
            }
        })
        .collect::<Result<()>>();
    // endregion: exec
    let elapsed = instant.elapsed();

    // Извлекаем ошибку, если она была
    match result {
        Err(e) => Err(e),
        Ok(_) => {
            let res_count = ws.packages.iter().flat_map(|it| &it.resources).count();
            info!(target: "Finished", "{res_count} resource(s) in {}", format_duration(elapsed));
            Ok(())
        }
    }
}

fn setup_cache(dir: &Path) -> Result<Cache> {
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
