use lib_label::LabelPattern;
use phase_evaluation::builder::EvalBuilder;

mod error;
pub use error::*;

pub struct FeatureFetchOptions {
    pub pattern: Vec<String>,
}

pub fn fetch(opts: FeatureFetchOptions) -> Result<()> {
    let pattern = LabelPattern::try_from(opts.pattern)?;
    let ws = phase_loading::load_workspace(pattern)?;
    {
        let graph = EvalBuilder::from_workspace(&ws)
            .fetch_remotes(true)
            .fetch_resources()
            .build()?;
        phase_evaluation::evaluate(ws, graph)?;
    }
    Ok(())
}
