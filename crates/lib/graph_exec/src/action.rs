//! # Action graph implementation

use crate::{
    NodeId, configured::ConfiguredExecutionGraph, unconfigured::UnconfiguredExecutionGraph,
};
use dashmap::DashMap;
use log::debug;
use std::{marker::PhantomData, sync::Arc};

/// An executable directed acyclic graph (DAG) of actions with typed inputs and outputs.
///
/// [`ActionGraph<T, E>`] represents a fully validated and configured execution graph
/// consisting of nodes that implement the [`Action<T, E>`] trait.
/// It ensures that all actions are executed in the correct topological order based
/// on their declared dependencies.
/// - `T` — the data type passed between actions.
/// - `E` — the error type that can be returned from action execution.
///
/// Use the [`ActionGraph::builder`] method to construct the graph using [`ActionGraphBuilder`],
/// then call [`ActionGraph::execute`] to run all nodes in dependency order.
pub struct ActionGraph<T, E, S>
where
    T: Clone + Send + Sync,
    E: Send + Sync,
    S: Clone + Send + Sync,
{
    graph: ConfiguredExecutionGraph<Box<dyn Action<T, E, S>>>,
}

impl<T, E, S> ActionGraph<T, E, S>
where
    T: Clone + Send + Sync,
    E: Send + Sync,
    S: Clone + Send + Sync,
{
    /// Creates a new builder for constructing an [`ActionGraph`].
    pub fn builder() -> ActionGraphBuilder<T, E, S> {
        ActionGraphBuilder::<T, E, S> {
            graph: UnconfiguredExecutionGraph::default(),
            _e: Default::default(),
        }
    }

    /// Executes the graph, evaluating all actions in dependency-respecting order.
    ///
    /// Consumes the configured graph and runs each action node by calling its [`Action::execute`]
    /// implementation. Nodes are executed in topological order according to their dependencies.
    /// When an action produces a result, that result is propagated as input to all dependent nodes.
    ///
    /// - If all actions complete successfully, returns `Ok(())`.
    /// - If any action returns an error (`Err(E)`), the execution stops and the error is returned.
    pub fn execute(self, state: S) -> Result<(), E> {
        let dependents = self.graph.dependents.clone();
        let providers: Arc<DashMap<NodeId, Vec<T>>> = Default::default();
        self.graph.execute(|id, node| {
            let inputs = providers
                .remove(&id)
                .map(|(_, vec)| vec)
                .unwrap_or_default();
            let ctx: ExecutionContext<T, S> = ExecutionContext {
                inputs,
                state: state.clone(),
            };

            match node.execute(ctx) {
                Ok(output) => {
                    if let Some(deps) = dependents.get(&id) {
                        for &dep_id in deps {
                            // Add outputs to the dependent actions
                            providers.entry(dep_id).or_default().push(output.clone());
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    debug!("Node failed: {}", node.diagnostics_info().name);
                    Err(e)
                }
            }
        })
    }

    /// Get inner representation of this graph
    ///
    /// For diagnostics and debug purposes
    pub fn into_inner(self) -> ConfiguredExecutionGraph<Box<dyn Action<T, E, S>>> {
        self.graph
    }
}

/// A builder for constructing an [`ActionGraph`] by adding actions and their dependencies.
///
/// The builder allows you to incrementally build an execution graph by adding actions
/// and specifying dependencies between them. Once complete, the [`ActionGraphBuilder::build`]
/// method validates and returns an executable [ActionGraph<T, E>].
/// - `T` — type of input and output values used in actions.
/// - `E` — error type that may be produced during graph execution.
///
/// See also:
/// - [`ActionGraphBuilder::add_action`]
/// - [`ActionGraphBuilder::add_dependency`]
/// - [`ActionGraphBuilder::build`]
/// - [`crate::graph_deps!`]
pub struct ActionGraphBuilder<T, E, S>
where
    T: Clone + Send + Sync,
    E: Send + Sync,
    S: Clone + Send + Sync,
{
    graph: UnconfiguredExecutionGraph<Box<dyn Action<T, E, S>>>,
    _e: PhantomData<E>,
}

/// A unique identifier for an action within an [`ActionGraph`].
///
/// [`ActionId`] is used to refer to nodes (actions) added to an [`ActionGraphBuilder`].
/// It allows users to establish dependencies between actions using readable and strongly
/// typed handles instead of raw node identifiers.
///
/// Used as input to [ActionGraphBuilder::add_dependency] to connect actions.
/// You can use it as `HashMap` or `HashSet` keys.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct ActionId(NodeId);

impl<T, E, S> ActionGraphBuilder<T, E, S>
where
    T: Clone + Send + Sync,
    E: Send + Sync,
    S: Clone + Send + Sync,
{
    /// Adds a new action node to the graph.
    ///
    /// This method registers a new `action` node in the graph and returns its unique [ActionId].
    /// Returns an [ActionId] that can later be used to define dependencies via [`add_dependency`]
    /// method or identify nodes during testing.
    pub fn add_action(&mut self, action: impl Action<T, E, S> + 'static) -> ActionId {
        ActionId(self.graph.add_node(Box::new(action)))
    }

    /// Declares a dependency between two actions in the graph.
    ///
    /// Defines that one action depends on the output of another.
    /// This will ensure that the `depends_on_what` action is executed before `what`.
    /// This method must only be called with valid [`ActionId`]'s that were previously
    /// returned by [`add_action`] method.
    pub fn add_dependency(&mut self, what: ActionId, depends_on_what: ActionId) {
        self.graph.add_dependency(what.0, depends_on_what.0);
    }

    /// Validates and constructs the executable [`ActionGraph`].
    ///
    /// Consumes the builder and performs validation of the internal dependency graph.
    /// This includes checking for cycles and performing topological sorting. If the graph is valid,
    /// it returns an [`ActionGraph<T, E>`] ready for execution.
    ///
    /// This is the final method called after all nodes and dependencies have been added.
    pub fn build(self) -> crate::unconfigured::Result<ActionGraph<T, E, S>> {
        Ok(ActionGraph::<T, E, S> {
            graph: self.graph.configure()?,
        })
    }
}

/// A trait representing a single executable unit in an action graph.
///
/// The [`Action<T, E>`] trait defines a unit of execution that can consume
/// input values of type `T` (provided by its dependencies) and return a result of type `T`
/// or an error of type `E`.
///
/// This trait must be implemented for all action nodes added to the graph.
/// Implementations must be thread-safe (Send + Sync) since actions may be executed concurrently.
pub trait Action<T, E, S>: Send + Sync {
    /// Executes the action logic with the provided input context.
    ///
    /// This method defines the core logic of an action node. It receives an [`ExecutionContext<T>`],
    /// which contains the outputs of all its dependencies (if any), and returns either a computed
    /// value of type `T` or an error of type `E`.
    ///
    /// This method is called automatically during graph execution in
    /// topological order determined by dependencies.
    fn execute(&self, ctx: ExecutionContext<T, S>) -> std::result::Result<T, E>;

    /// Returns diagnostics info for action
    ///
    /// The info is used in `figmagic aquery` output
    fn diagnostics_info(&self) -> ActionDiagnostics;
}

/// Diagnostics info for [`Action`]
///
/// This info is used in `figmagic aquery` output
#[cfg_attr(test, derive(Default))]
pub struct ActionDiagnostics {
    /// Name of the action
    pub name: String,
    /// Params which action wants to expose for debug
    pub params: Vec<(String, String)>,
}

/// Provides a node with the outputs of its dependencies during execution.
///
/// The context is passed into an action’s `execute()` method and contains the aggregated
/// outputs (`inputs`) from all dependency actions. This allows each action to reason about
/// the results of upstream computations.
pub struct ExecutionContext<T, S> {
    /// A vector with the outputs of dependent actions. Order is not guaranteed.
    pub inputs: Vec<T>,
    /// The state that the initiating party wants to transmit to all nodes
    pub state: S,
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use std::sync::Mutex;

    use super::*;
    use crate::graph_deps;

    #[test]
    fn run_single_valid_action__EXPECT__ok() {
        // Given
        struct SingleAction;
        impl Action<i32, String, ()> for SingleAction {
            fn execute(&self, ctx: ExecutionContext<i32, ()>) -> std::result::Result<i32, String> {
                assert!(ctx.inputs.is_empty());
                Ok(0)
            }
            fn diagnostics_info(&self) -> ActionDiagnostics {
                Default::default()
            }
        }
        let mut graph = ActionGraph::builder();
        let _ = graph.add_action(SingleAction);
        let graph = graph.build().unwrap();

        // When
        let result = graph.execute(());

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn run_single_invalid_action__EXPECT__err() {
        // Given
        struct SingleAction;
        impl Action<i32, String, ()> for SingleAction {
            fn execute(&self, ctx: ExecutionContext<i32, ()>) -> std::result::Result<i32, String> {
                assert!(ctx.inputs.is_empty());
                Err("test error".into())
            }
            fn diagnostics_info(&self) -> ActionDiagnostics {
                Default::default()
            }
        }
        let mut graph = ActionGraph::builder();
        let _ = graph.add_action(SingleAction);
        let graph = graph.build().unwrap();

        // When
        let result = graph.execute(());

        // Then
        assert_eq!(Err("test error".to_string()), result);
    }

    #[test]
    fn run_multiple_valid_actions__EXPECT__correct_result() {
        // Given
        struct InitAction(i32);
        struct MultiplyAction(i32, Arc<Mutex<i32>>);
        impl Action<i32, String, ()> for InitAction {
            fn execute(&self, ctx: ExecutionContext<i32, ()>) -> std::result::Result<i32, String> {
                assert!(ctx.inputs.is_empty());
                Ok(self.0)
            }
            fn diagnostics_info(&self) -> ActionDiagnostics {
                Default::default()
            }
        }
        impl Action<i32, String, ()> for MultiplyAction {
            fn execute(&self, ctx: ExecutionContext<i32, ()>) -> std::result::Result<i32, String> {
                assert_eq!(Some(&2), ctx.inputs.first());
                let result = ctx.inputs.first().unwrap() * self.0;
                *self.1.lock().unwrap() = result;
                Ok(result)
            }
            fn diagnostics_info(&self) -> ActionDiagnostics {
                Default::default()
            }
        }
        let mut graph = ActionGraph::builder();
        let action_init = graph.add_action(InitAction(2));
        let calc_result: Arc<Mutex<i32>> = Default::default();
        let action_multiply = graph.add_action(MultiplyAction(4, calc_result.clone()));
        graph_deps! { graph, action_multiply => action_init };
        let graph = graph.build().unwrap();

        // When
        let result = graph.execute(());

        // Then
        assert!(result.is_ok());
        assert_eq!(8, *calc_result.lock().unwrap());
    }
}
