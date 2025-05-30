use lib_label::LabelPattern;

mod error;
pub use error::*;
use phase_evaluation::EvalArgs;

pub struct FeatureFetchOptions {
    pub pattern: Vec<String>,
}

pub fn fetch(opts: FeatureFetchOptions) -> Result<()> {
    let pattern = LabelPattern::try_from(opts.pattern)?;
    let ws = phase_loading::load_workspace(pattern)?;
    {
        phase_evaluation::evaluate(
            ws,
            EvalArgs {
                fetch: true,
                ..Default::default()
            },
        )?;
    }
    Ok(())
}
