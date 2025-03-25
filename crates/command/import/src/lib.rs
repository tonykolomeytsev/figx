use lib_label::LabelPattern;
use phase_evaluation::builder::EvalBuilder;

mod error;
pub use error::*;

pub struct FeatureImportOptions {
    pub pattern: Vec<String>,
}

pub fn import(opts: FeatureImportOptions) -> Result<()> {
    let pattern = LabelPattern::try_from(opts.pattern)?;
    let ws = phase_loading::load_workspace(pattern)?;
    {
        let graph = EvalBuilder::from_workspace(&ws)
            .fetch_remotes(false)
            .fetch_resources()
            .transform_resources()
            .materialize_resources()
            .build()?;
        phase_evaluation::evaluate(ws, graph)?;
    }
    Ok(())
}
