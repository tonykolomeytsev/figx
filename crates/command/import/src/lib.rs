use lib_label::LabelPattern;

mod error;
pub use error::*;
use lib_metrics::Metrics;
use log::warn;
use phase_evaluation::EvalArgs;

pub struct FeatureImportOptions {
    pub pattern: Vec<String>,
    pub refetch: bool,
    pub concurrency: usize,
}

pub fn import(opts: FeatureImportOptions) -> Result<()> {
    // region: metrics
    let metrics = Metrics::default();
    let full_duration = metrics.duration("figx_full_duration");
    let loading_duration = metrics.duration("figx_loading_duration");
    let full_duration = full_duration.record();
    // endregion: metrics

    let loading_duration = loading_duration.record();
    let pattern = LabelPattern::try_from(opts.pattern)?;
    let ws = phase_loading::load_workspace(pattern, false)?;
    let cache_dir = ws.context.cache_dir.clone();
    drop(loading_duration);
    {
        phase_evaluation::evaluate(
            ws,
            EvalArgs {
                refetch: opts.refetch,
                concurrency: opts.concurrency,
                metrics: metrics.clone(),
                ..Default::default()
            },
        )?;
    }

    drop(full_duration);
    if let Err(e) = metrics.export_as_prometheus(
        Some(&[("command", "import")]),
        &cache_dir.join("metrics.prom"),
    ) {
        warn!("Unable to save metrics: {e}")
    }
    Ok(())
}
