//! # Configured execution graph
//!
//! This module provides the implementation of [`ConfiguredExecutionGraph`].
//!
//! [`ConfiguredExecutionGraph`] is created from a validated [`UnconfiguredExecutionGraph`]
//! and provides the ability to execute all nodes in the order determined by their dependencies.
//!
//! # Examples
//!
//! Execute graph:
//!
//! ```
//! # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
//! # use lib_graph_exec::graph_deps;
//! # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
//! # let first_node = graph.add_node(1);
//! # let second_node = graph.add_node(2);
//! # let third_node = graph.add_node(3);
//! # // Arrow '=>' direction shows dependency direction (what => depends_on_what)
//! # graph_deps! { graph, first_node => second_node => third_node };
//! let graph = graph.configure().unwrap();
//! graph.execute(|node_id, node_val| {
//!     eprintln!("Node with value '{node_val}' executed!");
//!     Ok::<(), i32>(())
//! });
//! ```
//!
//! Code above will print:
//!
//! ```text
//! Node with value '3' executed!
//! Node with value '2' executed!
//! Node with value '1' executed!
//! ```

use crate::NodeId;
use dashmap::{DashMap, DashSet};
use log::{debug, trace};
use ordermap::OrderMap;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

/// A validated, executable action graph with deterministic, dependency-respecting parallel execution.
///
/// ConfiguredExecutionGraph represents an execution-ready, topologically sorted
/// directed acyclic graph (DAG) constructed from a previously validated [`UnconfiguredExecutionGraph`].
/// It holds all necessary internal state to run the graph's nodes in an order that respects their
/// declared dependencies.
///
/// Each node is executed exactly once, and only after all of its dependencies have completed successfully.
/// Node execution is performed in parallel using a thread pool. Users provide a function that is applied
/// to each node's value during execution. If any node fails (i.e., the function returns an error),
/// execution halts and the error is propagated to the caller.
///
/// This type is thread-safe.
///
/// # Examples
///
/// Execute graph:
///
/// ```
/// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
/// # use lib_graph_exec::graph_deps;
/// # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
/// # let first_node = graph.add_node(1);
/// # let second_node = graph.add_node(2);
/// # let third_node = graph.add_node(3);
/// # // Arrow '=>' direction shows dependency direction (what => depends_on_what)
/// # graph_deps! { graph, first_node => second_node => third_node };
/// let graph = graph.configure().unwrap();
/// graph.execute(|node_id, node_val| {
///     eprintln!("Node with value '{node_val}' executed!");
///     Ok::<(), i32>(())
/// });
/// ```
///
/// Code above will print:
///
/// ```text
/// Node with value '3' executed!
/// Node with value '2' executed!
/// Node with value '1' executed!
/// ```
#[cfg_attr(test, derive(Debug))]
pub struct ConfiguredExecutionGraph<T: Send + Sync> {
    /// Topologically sorted nodes
    pub nodes: Arc<DashMap<NodeId, Node<T>>>,
    /// Adjacency list mapping node IDs to their dependents (nodes that depend on them)
    pub dependents: HashMap<NodeId, Vec<NodeId>>,
    /// Pre-computed incoming edge counts for each node
    pub incoming_edge_counts: OrderMap<NodeId, usize>,
}

/// A single node within the configured execution graph.
///
/// [`Node<T>`] represents an individual unit of execution within the graph.
/// It stores a unique node identifier ([`NodeId`]) and the user-provided data
/// of type `T` associated with that node. Nodes are not meant to be constructed
/// manually by the user; they are created internally when the graph is built.
///
/// This struct is primarily used internally during graph execution and is typically
/// not manipulated directly by library users.
#[non_exhaustive]
#[cfg_attr(test, derive(Debug))]
pub struct Node<T> {
    /// Unique node identifier
    pub id: NodeId,
    /// Owned node data
    pub data: T,
}

impl<T: Send + Sync> ConfiguredExecutionGraph<T> {
    /// Executes all nodes in the graph in dependency-respecting order.
    ///
    /// This method consumes the [`ConfiguredExecutionGraph`] and runs the provided
    /// function on each node's data. The execution respects the topological order of the graph:
    /// a node is not executed until all of its dependencies have completed successfully.
    ///
    /// If the function returns an error for any node, execution halts and that error is returned.
    ///
    /// This method is the core mechanism for executing a dependency graph.
    /// # Examples
    ///
    /// Execute graph:
    ///
    /// ```
    /// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
    /// # use lib_graph_exec::graph_deps;
    /// # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
    /// # let first_node = graph.add_node(1);
    /// # let second_node = graph.add_node(2);
    /// # let third_node = graph.add_node(3);
    /// # // Arrow '=>' direction shows dependency direction (what => depends_on_what)
    /// # graph_deps! { graph, first_node => second_node => third_node };
    /// let graph = graph.configure().unwrap();
    /// graph.execute(|node_id, node_val| {
    ///     eprintln!("Node with value '{node_val}' executed!");
    ///     Ok::<(), i32>(())
    /// });
    /// ```
    pub fn execute<E: Send>(
        self,
        exec: impl Fn(NodeId, T) -> std::result::Result<(), E> + Send + Sync,
    ) -> Result<(), E> {
        let remaining_deps = Arc::new(Mutex::new(self.incoming_edge_counts.clone()));
        let error = Arc::new(Mutex::new(None));
        let exec = Arc::new(exec);

        // Track completed nodes to know when we're done
        let completed = Arc::new(DashSet::<NodeId>::new());
        let total_nodes = self.nodes.len();

        rayon::scope(|s| {
            // Channel for communicating ready nodes to workers
            let (ready_sender, ready_receiver): (Sender<Option<NodeId>>, Receiver<Option<NodeId>>) =
                channel();

            // Initial ready nodes
            for node_id in self.nodes.iter().map(|it| *it.key()) {
                if *remaining_deps
                    .lock()
                    .unwrap()
                    .get(&node_id)
                    .expect("edge counts present for all node ids")
                    == 0
                {
                    ready_sender.send(Some(node_id)).unwrap();
                }
            }

            // Process nodes until all are done or error occurs
            while completed.len() < total_nodes {
                let node_id = match ready_receiver.recv() {
                    Ok(Some(id)) => id,
                    Err(_) | Ok(None) => break, // Channel closed, all workers finished
                };
                trace!("Received node {node_id:?} for execution");

                // Check for existing errors
                if error.lock().unwrap().is_some() {
                    break;
                }

                let node = {
                    self.nodes
                        .remove(&node_id)
                        .expect("each node executes only once")
                        .1
                };
                // Mark as processing (important for tracking)
                completed.insert(node_id);

                // Process node in parallel
                // Clone necessary shared state
                let remaining_deps = Arc::clone(&remaining_deps);
                let dependents = self.dependents.clone();
                let error = Arc::clone(&error);
                let exec = exec.clone();

                let ready_sender = ready_sender.clone();
                s.spawn(move |_| match exec(node.id, node.data) {
                    Ok(()) => {
                        trace!("Node {node_id:?} executed successfully");
                        if let Some(deps) = dependents.get(&node_id) {
                            let mut remaining = remaining_deps.lock().unwrap();
                            for &dep_id in deps {
                                let count = remaining.get_mut(&dep_id).unwrap();
                                *count -= 1;
                                if *count == 0 {
                                    // If channel already closed - some other action failed
                                    let _ = ready_sender.send(Some(dep_id));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Node {node_id:?} execution failed");
                        let _ = ready_sender.send(None);
                        *error.lock().unwrap() = Some(e);
                    }
                });
            }
        });

        error.lock().unwrap().take().map(Err).unwrap_or(Ok(()))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use std::sync::{Arc, Mutex};

    use crate::graph_deps;
    use crate::unconfigured::UnconfiguredExecutionGraph;

    #[test]
    fn create_valid_adg_and_run() {
        // Given
        let mut graph = UnconfiguredExecutionGraph::<&str>::default();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        graph_deps! { graph, a => b => c => d };
        let graph = graph.configure().unwrap();

        // When
        let result: Result<(), &str> = graph.execute(|_, node| {
            eprintln!("Executing node: {node}");
            Ok(())
        });

        // Then
        assert!(result.is_ok())
    }

    #[test]
    fn execute_adg_with_deps__EXPECT__valid_execution_order() {
        // Given
        let mut graph = UnconfiguredExecutionGraph::<&str>::default();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        let d = graph.add_node("D");
        graph_deps! { graph, a => b => c => d };
        let graph = graph.configure().unwrap();
        let expected_exec_order: &[&str] = &["D", "C", "B", "A"];

        // When
        let exec_order: Arc<Mutex<Vec<String>>> = Default::default();
        graph
            .execute::<()>(|_, node| {
                let exec_order = Arc::clone(&exec_order);
                exec_order.lock().unwrap().push(node.to_string());
                Ok(())
            })
            .unwrap();

        // Then
        assert_eq!(expected_exec_order, *exec_order.lock().unwrap());
    }
}
