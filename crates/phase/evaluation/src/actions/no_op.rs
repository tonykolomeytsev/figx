use lib_cache::CacheKey;
use lib_graph_exec::action::{Action, ActionDiagnostics, ExecutionContext};
use lib_label::Label;

use crate::{Error, EvalState, Result};

pub struct NoOpAction {
    pub label: Label,
}

impl Action<CacheKey, Error, EvalState> for NoOpAction {
    fn execute(&self, _ctx: ExecutionContext<CacheKey, EvalState>) -> Result<CacheKey> {
        unreachable!("Will never been executed")
    }

    fn diagnostics_info(&self) -> ActionDiagnostics {
        ActionDiagnostics {
            name: self.label.to_string(),
            params: Vec::new(),
        }
    }
}
