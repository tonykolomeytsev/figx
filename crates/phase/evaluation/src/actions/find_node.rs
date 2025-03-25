use super::ExecutionContextExt;
use super::fetch_remote::RemoteMetadata;
use super::volatile_action;
use crate::Error;
use crate::EvalState;
use crate::Result;
use lib_cache::CacheKey;
use lib_graph_exec::action::Action;
use lib_graph_exec::action::ActionDiagnostics;
use lib_graph_exec::action::ExecutionContext;
use lib_label::Label;
use log::debug;

pub struct FindNodeAction {
    pub label: Label,
    pub node_name: String,
}

impl Action<CacheKey, Error, EvalState> for FindNodeAction {
    fn execute(&self, ctx: ExecutionContext<CacheKey, EvalState>) -> Result<CacheKey> {
        let fetch_remote_cache_key = ctx.single_input()?;
        let stable_cache_key = CacheKey::builder()
            .write(fetch_remote_cache_key.as_ref())
            .write_str(&self.node_name)
            .build();

        volatile_action(&ctx.state.cache, stable_cache_key, true, |cache_key| {
            self.find_node_id_impl(fetch_remote_cache_key, cache_key, &ctx.state)
        })
    }

    fn diagnostics_info(&self) -> ActionDiagnostics {
        ActionDiagnostics {
            name: "Find node by name in Figma file".to_string(),
            params: vec![("node_name".to_string(), self.node_name.clone())],
        }
    }
}

impl FindNodeAction {
    pub const TAG: u8 = 0x01;

    fn find_node_id_impl(
        &self,
        fetch_remote_cache_key: &CacheKey,
        stable_cache_key: CacheKey,
        state: &EvalState,
    ) -> Result<CacheKey> {
        let FindNodeAction { label, node_name } = &self;

        let ui_state = state.renderer.get_handle();
        ui_state.set_state(lib_pretty::State::Fetching(label.to_string()));

        debug!("Seeking id for node with name '{node_name}'...");
        let remote_metadata = state
            .cache
            .require::<RemoteMetadata>(fetch_remote_cache_key)?;
        let node = remote_metadata
            .name_to_node
            .get(node_name)
            .ok_or(Error::FindNode {
                node_name: self.node_name.clone(),
            })?;

        let volatile_cache_key = CacheKey::builder()
            .set_tag(Self::TAG)
            .write(stable_cache_key.as_ref())
            .write_str(node.id.as_ref())
            .write_u64(node.hash)
            .build();

        state.cache.put(&volatile_cache_key, &node.id)?;
        Ok(volatile_cache_key)
    }
}
