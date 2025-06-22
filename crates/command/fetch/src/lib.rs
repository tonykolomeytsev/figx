use lib_label::LabelPattern;

mod error;
pub use error::*;
use lib_metrics::Metrics;
use phase_evaluation::EvalArgs;

pub struct FeatureFetchOptions {
    pub pattern: Vec<String>,
    pub concurrency: usize,
}

pub fn fetch(opts: FeatureFetchOptions) -> Result<()> {
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
                fetch: true,
                concurrency: opts.concurrency,
                metrics: metrics.clone(),
                ..Default::default()
            },
        )?;
    }

    drop(full_duration);
    metrics.export_as_prometheus(
        Some(&[("command", "fetch")]),
        &cache_dir.join("metrics.prom"),
    );
    Ok(())
}
