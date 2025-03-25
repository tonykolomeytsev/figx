use lib_cache::{Cache, CacheKey};
use lib_figma::FigmaApi;
use lib_pretty::StateRenderer;
use log::{debug, info, trace};
use phase_loading::Workspace;
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

mod actions;
pub mod builder;
mod error;
mod hashing;
pub use actions::*;
pub use error::*;
pub use hashing::*;

pub type ActionGraph = lib_graph_exec::action::ActionGraph<CacheKey, Error, EvalState>;

#[derive(Clone)]
pub struct EvalState {
    pub figma_api: FigmaApi,
    pub renderer: Arc<StateRenderer>,
    pub cache: Cache,
}

pub fn evaluate(ws: Workspace, graph: ActionGraph) -> Result<()> {
    info!(
        "Requested resources: {}",
        ws.packages.iter().flat_map(|it| &it.resources).count()
    );
    let instant = Instant::now();
    let eval_state = init_eval_state(&ws)?;
    let result = graph.execute(eval_state);
    let elapsed = instant.elapsed();
    info!("Time elapsed: {}", format_duration(elapsed));

    // Извлекаем ошибку, если она была
    match result {
        Err(e) => Err(e),
        Ok(_) => Ok(()),
    }
}

fn setup_cache(dir: &Path) -> Result<Cache> {
    trace!("Ensuring all dirs to cache DB exists...");
    std::fs::create_dir_all(dir)?;
    debug!("Loading cache...");
    Ok(Cache::new(dir)?)
}

fn init_eval_state(ws: &Workspace) -> Result<EvalState> {
    Ok(EvalState {
        figma_api: Default::default(),
        renderer: Default::default(),
        cache: setup_cache(&ws.context.cache_dir)?,
    })
}

fn format_duration(duration: Duration) -> String {
    let total_millis = duration.as_millis();

    if total_millis < 1000 {
        return format!("{} s", total_millis as f32 / 1000f32);
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
