use lib_label::LabelPattern;

mod error;
pub use error::*;
use phase_evaluation::EvalArgs;

pub struct FeatureImportOptions {
    pub pattern: Vec<String>,
    pub refetch: bool,
}

pub fn import(opts: FeatureImportOptions) -> Result<()> {
    let pattern = LabelPattern::try_from(opts.pattern)?;
    let ws = phase_loading::load_workspace(pattern)?;
    {
        phase_evaluation::evaluate(
            ws,
            EvalArgs {
                refetch: opts.refetch,
                ..Default::default()
            },
        )?;
    }
    Ok(())
}
