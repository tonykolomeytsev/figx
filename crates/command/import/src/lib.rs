use lib_label::LabelPattern;

mod error;
pub use error::*;
use phase_evaluation::EvalArgs;

pub struct FeatureImportOptions {
    pub pattern: Vec<String>,
    pub refetch: bool,
    pub concurrency: usize,
}

pub fn import(opts: FeatureImportOptions) -> Result<()> {
    let pattern = LabelPattern::try_from(opts.pattern)?;
    let ws = phase_loading::load_workspace(pattern, false)?;
    {
        phase_evaluation::evaluate(
            ws,
            EvalArgs {
                refetch: opts.refetch,
                concurrency: opts.concurrency,
                ..Default::default()
            },
        )?;
    }
    Ok(())
}
