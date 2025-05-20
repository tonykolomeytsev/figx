//! # Action graph implementation

use crate::{
    NodeId, configured::ConfiguredExecutionGraph, unconfigured::UnconfiguredExecutionGraph,
};
use dashmap::DashMap;
use log::debug;
use std::{hash::Hash, marker::PhantomData, sync::Arc};

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
pub struct ActionGraph<P, S, E> {
    graph: ConfiguredExecutionGraph<Action<P, S, E>>,
}

impl<P, S, E> ActionGraph<P, S, E>
where
    E: Send + Sync,
    S: Send + Sync,
    P: Clone + Send + Sync,
{
    /// Creates a new builder for constructing an [`ActionGraph`].
    pub fn builder() -> ActionGraphBuilder<P, S, E> {
        ActionGraphBuilder::<P, S, E> {
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
        let providers: Arc<DashMap<NodeId, Vec<P>>> = Default::default();
        self.graph.execute(|id, node| {
            let inputs = providers
                .remove(&id)
                .map(|(_, vec)| vec)
                .unwrap_or_default();
            let mut ctx: AnalysisContext<P, S, E> = AnalysisContext {
                inputs: &inputs,
                state: &state,
                action: None,
            };
            let analysis_result = node.analyze.analyze(&mut ctx);
            // analyze
            match (analysis_result, ctx.action) {
                (Ok(()), None) => Ok(()),
                (Ok(()), Some(action)) => match action() {
                    Ok(provider) => {
                        if let Some(deps) = dependents.get(&id) {
                            for &dep_id in deps {
                                // Add outputs to the dependent actions
                                providers.entry(dep_id).or_default().push(provider.clone());
                            }
                        }
                        Ok(())
                    }
                    Err(e) => {
                        debug!("Node action failed: {}", node.analyze.meta().name);
                        Err(e)
                    }
                },
                (Err(e), _) => {
                    debug!("Node analysis failed: {}", node.analyze.meta().name);
                    Err(e)
                }
            }
        })
    }

    /// Get inner representation of this graph
    ///
    /// For diagnostics and debug purposes
    pub fn into_inner(self) -> ConfiguredExecutionGraph<Action<P, S, E>> {
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
pub struct ActionGraphBuilder<E, S, P> {
    graph: UnconfiguredExecutionGraph<Action<E, S, P>>,
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

impl<P, S, E> ActionGraphBuilder<P, S, E>
where
    E: Send + Sync,
    S: Send + Sync,
    P: Clone + Send + Sync,
{
    /// Adds a new action node to the graph.
    ///
    /// This method registers a new `action` node in the graph and returns its unique [ActionId].
    /// Returns an [ActionId] that can later be used to define dependencies via [`add_dependency`]
    /// method or identify nodes during testing.
    pub fn add_action(
        &mut self,
        action: impl ActionImpl<Providers = P, State = S, Error = E> + 'static,
    ) -> ActionId {
        let action = Action {
            digest: action.digest(),
            analyze: Box::new(action),
        };
        ActionId(self.graph.add_node(action))
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
    pub fn build(self) -> crate::unconfigured::Result<ActionGraph<P, S, E>> {
        Ok(ActionGraph::<P, S, E> {
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
pub struct Action<P, S, E> {
    digest: u64,
    analyze: Box<dyn ActionImpl<Providers = P, State = S, Error = E>>,
}

impl<P, S, E> Eq for Action<P, S, E> {}
impl<P, S, E> PartialEq for Action<P, S, E> {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
    }
}
impl<P, S, E> Hash for Action<P, S, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.digest);
    }
}

/// Analysis
pub struct AnalysisContext<'a, P, S, E> {
    /// A vector with the outputs of dependent actions. Order is not guaranteed.
    pub inputs: &'a [P],
    /// The state that the initiating party wants to transmit to all nodes
    pub state: &'a S,
    /// Executing action
    action: Option<Box<dyn FnOnce() -> std::result::Result<P, E>>>,
}

impl<'a, P, S, E> AnalysisContext<'a, P, S, E> {
    /// Action
    pub fn action(&mut self, action: impl FnOnce() -> std::result::Result<P, E> + 'static) {
        self.action = Some(Box::new(action))
    }
}

/// No docs
pub trait ActionImpl: Send + Sync {
    /// No docs
    type Providers;
    /// No docs
    type State;
    /// No docs
    type Error;

    /// No docs
    fn digest(&self) -> u64;

    /// No docs
    fn analyze<'a>(
        &self,
        ctx: &mut AnalysisContext<'a, Self::Providers, Self::State, Self::Error>,
    ) -> std::result::Result<(), Self::Error>;

    /// No docs
    fn meta(&self) -> ActionMeta;
}

/// Diagnostics info for [`Action`]
///
/// This info is used in `figmagic aquery` output
#[cfg_attr(test, derive(Default))]
pub struct ActionMeta {
    /// Name of the action
    pub name: &'static str,
    /// Params which action wants to expose for debug
    pub params: Vec<(&'static str, String)>,
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {

    use super::*;
    use crate::graph_deps;

    #[test]
    fn run_single_valid_action__EXPECT__ok() {
        // Given
        struct SingleAction;
        impl ActionImpl for SingleAction {
            type Providers = i32;
            type State = ();
            type Error = &'static str;

            fn digest(&self) -> u64 {
                42
            }

            fn meta(&self) -> ActionMeta {
                Default::default()
            }

            fn analyze<'a>(
                &self,
                _ctx: &mut AnalysisContext<'a, Self::Providers, Self::State, Self::Error>,
            ) -> std::result::Result<(), Self::Error> {
                Ok(())
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
        impl ActionImpl for SingleAction {
            type Providers = i32;
            type State = ();
            type Error = &'static str;

            fn digest(&self) -> u64 {
                42
            }

            fn analyze<'a>(
                &self,
                _ctx: &mut AnalysisContext<'a, Self::Providers, Self::State, Self::Error>,
            ) -> std::result::Result<(), Self::Error> {
                Err("test error")
            }

            fn meta(&self) -> ActionMeta {
                Default::default()
            }
        }
        let mut graph = ActionGraph::builder();
        let _ = graph.add_action(SingleAction);
        let graph = graph.build().unwrap();

        // When
        let result = graph.execute(());

        // Then
        assert_eq!(Err("test error"), result);
    }

    #[test]
    fn run_multiple_valid_actions__EXPECT__correct_result() {
        // Given
        struct TestState(Arc<std::sync::Mutex<i32>>);
        struct InitAction(i32);
        struct MultiplyAction(i32);
        impl ActionImpl for InitAction {
            type Providers = i32;
            type State = TestState;
            type Error = &'static str;

            fn digest(&self) -> u64 {
                self.0 as u64
            }

            fn analyze<'a>(
                &self,
                ctx: &mut AnalysisContext<'a, Self::Providers, Self::State, Self::Error>,
            ) -> std::result::Result<(), Self::Error> {
                let InitAction(number) = *self;
                ctx.action(move || Ok(number));
                Ok(())
            }

            fn meta(&self) -> ActionMeta {
                Default::default()
            }
        }
        impl ActionImpl for MultiplyAction {
            type Providers = i32;
            type State = TestState;
            type Error = &'static str;

            fn digest(&self) -> u64 {
                self.0 as u64
            }

            fn analyze<'a>(
                &self,
                ctx: &mut AnalysisContext<'a, Self::Providers, Self::State, Self::Error>,
            ) -> std::result::Result<(), Self::Error> {
                *ctx.state.0.lock().unwrap() = ctx.inputs.first().unwrap() * self.0;
                Ok(())
            }

            fn meta(&self) -> ActionMeta {
                Default::default()
            }
        }
        let mut graph = ActionGraph::builder();
        let action_init = graph.add_action(InitAction(2));
        let action_multiply = graph.add_action(MultiplyAction(4));
        graph_deps! { graph, action_multiply => action_init };
        let graph = graph.build().unwrap();

        // When
        let calc_result: Arc<std::sync::Mutex<i32>> = Default::default();
        let result = graph.execute(TestState(calc_result.clone()));

        // Then
        assert!(result.is_ok());
        assert_eq!(8, *calc_result.lock().unwrap());
    }

    #[test]
    fn run_sum_actions__EXPECT__correct_result() {
        // Given
        struct TestState(Arc<std::sync::Mutex<i32>>);
        struct InitAction(i32);
        struct SumAction;
        impl ActionImpl for InitAction {
            type Providers = i32;
            type State = TestState;
            type Error = &'static str;

            fn digest(&self) -> u64 {
                self.0 as u64
            }

            fn analyze<'a>(
                &self,
                ctx: &mut AnalysisContext<'a, Self::Providers, Self::State, Self::Error>,
            ) -> std::result::Result<(), Self::Error> {
                let InitAction(number) = *self;
                ctx.action(move || Ok(number));
                Ok(())
            }

            fn meta(&self) -> ActionMeta {
                Default::default()
            }
        }
        impl ActionImpl for SumAction {
            type Providers = i32;
            type State = TestState;
            type Error = &'static str;

            fn digest(&self) -> u64 {
                999
            }

            fn analyze<'a>(
                &self,
                ctx: &mut AnalysisContext<'a, Self::Providers, Self::State, Self::Error>,
            ) -> std::result::Result<(), Self::Error> {
                *ctx.state.0.lock().unwrap() = ctx.inputs.iter().sum();
                Ok(())
            }

            fn meta(&self) -> ActionMeta {
                Default::default()
            }
        }
        let mut graph = ActionGraph::builder();
        let action_init1 = graph.add_action(InitAction(1));
        let action_init2 = graph.add_action(InitAction(2));
        let action_init3 = graph.add_action(InitAction(3));
        let action_init4 = graph.add_action(InitAction(4));
        let action_multiply = graph.add_action(SumAction);
        graph_deps! { graph, action_multiply => action_init1 };
        graph_deps! { graph, action_multiply => action_init2 };
        graph_deps! { graph, action_multiply => action_init3 };
        graph_deps! { graph, action_multiply => action_init4 };
        let graph = graph.build().unwrap();

        // When
        let calc_result: Arc<std::sync::Mutex<i32>> = Default::default();
        let result = graph.execute(TestState(calc_result.clone()));

        // Then
        assert!(result.is_ok());
        assert_eq!(10, *calc_result.lock().unwrap());
    }
}
