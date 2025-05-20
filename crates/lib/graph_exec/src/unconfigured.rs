//! # Unconfigured execution graph
//!
//! See also [`UnconfiguredExecutionGraph`].
//!
//! This module provides the implementation of [`UnconfiguredExecutionGraph`],
//! which can be seen as a builder for [`ConfiguredExecutionGraph`].
//!
//! [`UnconfiguredExecutionGraph`] is unaware of how the graph will be executed;
//! its purpose is to construct the final execution graph structure. It is responsible for
//! validating the graph, performing topological sorting, and detecting cycles.
//!
//! Also see [`graph_deps!`] macro.
//!
//! # Examples
//!
//! Creating graph:
//!
//! ```
//! # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
//! let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
//! ```
//!
//! Adding nodes to the graph:
//!
//! ```
//! # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
//! # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
//! let first_node = graph.add_node(1);
//! let second_node = graph.add_node(2);
//! let third_node = graph.add_node(3);
//! ```
//!
//! Adding dependencies between the nodes:
//!
//! ```
//! # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
//! # use lib_graph_exec::graph_deps;
//! # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
//! # let first_node = graph.add_node(1);
//! # let second_node = graph.add_node(2);
//! # let third_node = graph.add_node(3);
//! // Arrow '=>' direction shows dependency direction (what => depends_on_what)
//! graph_deps! { graph, first_node => second_node => third_node };
//! ```

use dashmap::DashMap;
use log::debug;
use ordermap::{OrderMap, OrderSet};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::{Debug, Display},
    hash::Hash,
    sync::Arc,
};

use crate::{
    NodeId,
    configured::{ConfiguredExecutionGraph, Node},
};

/// A mutable builder for defining an acyclic action dependency graph before execution.
///
/// UnconfiguredExecutionGraph is a utility type used to define the structure of a directed
/// acyclic graph (DAG) before it is executed. It is responsible for managing graph construction,
/// node registration, and dependency declarations.
///
/// This graph is not executable on its own. Instead, it serves as a builder for a [`ConfiguredExecutionGraph`],
/// which is produced by calling [`UnconfiguredExecutionGraph::configure()`] once the structure
/// is complete and valid.
///
/// # Examples
///
/// Creating graph:
///
/// ```
/// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
/// let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
/// ```
///
/// Adding nodes to the graph:
///
/// ```
/// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
/// # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
/// let first_node = graph.add_node(1);
/// let second_node = graph.add_node(2);
/// let third_node = graph.add_node(3);
/// ```
///
/// Adding dependencies between the nodes:
///
/// ```
/// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
/// # use lib_graph_exec::graph_deps;
/// # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
/// # let first_node = graph.add_node(1);
/// # let second_node = graph.add_node(2);
/// # let third_node = graph.add_node(3);
/// // Arrow '=>' direction shows dependency direction (what => depends_on_what)
/// graph_deps! { graph, first_node => second_node => third_node };
/// ```
#[cfg_attr(test, derive(Debug))]
pub struct UnconfiguredExecutionGraph<T: Send + Sync + Eq + Hash> {
    nodes: OrderSet<T>,
    direct_deps: HashMap<NodeId, HashSet<NodeId>>,
    invert_deps: HashMap<NodeId, HashSet<NodeId>>,

    /// OrderMap here is for making toposort deterministic
    in_degree: OrderMap<NodeId, Degree>,
    out_degree: OrderMap<NodeId, Degree>,
}

impl<T: Send + Sync + Eq + Hash> Default for UnconfiguredExecutionGraph<T> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
            direct_deps: Default::default(),
            invert_deps: Default::default(),
            in_degree: Default::default(),
            out_degree: Default::default(),
        }
    }
}

type Degree = usize;

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*{:?}", self.0)
    }
}

// region: Error handling

/// Return type for [`lib_graph_exec::unconfigured`] module
pub type Result<T> = std::result::Result<T, Error>;
/// Error type for [`lib_graph_exec::unconfigured`] module
pub type Error = UnconfiguredExecutionGraphError;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
/// Error type returned during validation of an unconfigured execution graph.
pub enum UnconfiguredExecutionGraphError {
    /// Indicates that a cycle was detected in the graph.
    GraphHasCycle {
        /// Sequence of node IDs forming the cycle.
        cycle: Vec<NodeId>,
    },
}

impl std::error::Error for UnconfiguredExecutionGraphError {}
impl Display for UnconfiguredExecutionGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GraphHasCycle { cycle } => write!(f, "error: graph has cycle: {:?}", cycle),
        }
    }
}

// endregion: Error handling

impl<T: Send + Sync + Eq + Hash> UnconfiguredExecutionGraph<T> {
    /// Adds a new node to the unconfigured execution graph and returns its unique identifier.
    ///
    /// This method adds a new `node` containing the data node of type `T` to the
    /// [`UnconfiguredExecutionGraph`]. Each node is assigned a unique [`NodeId`] upon insertion.
    /// The `node` is not yet part of the executable graph until the graph is configured.
    ///
    /// The [`NodeId`] returned by this method is used to reference the node when
    /// adding dependencies later.
    ///
    /// # Examples
    ///
    /// Adding nodes to the graph:
    ///
    /// ```
    /// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
    /// # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
    /// let first_node = graph.add_node(1);
    /// let second_node = graph.add_node(2);
    /// let third_node = graph.add_node(3);
    /// ```
    pub fn add_node(&mut self, node: T) -> NodeId {
        let (idx, _) = self.nodes.insert_full(node);
        let id = NodeId(idx);
        self.in_degree.entry(id).or_insert(0);
        self.out_degree.entry(id).or_insert(0);
        id
    }

    /// Declares a dependency between two nodes in the unconfigured execution graph.
    ///
    /// This method establishes a directional dependency between two nodes in the graph.
    /// Specifically, it sets that `what` depends on `depends_on_what`, meaning `what`
    /// will only be executed after `depends_on_what` has been successfully completed.
    ///
    /// Dependencies are important for determining the correct order in which nodes
    /// will be executed in the final [`ConfiguredExecutionGraph`].
    ///
    /// # Examples
    ///
    /// Adding dependencies between the nodes:
    ///
    /// ```
    /// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
    /// # use lib_graph_exec::graph_deps;
    /// # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
    /// # let first_node = graph.add_node(1);
    /// # let second_node = graph.add_node(2);
    /// # let third_node = graph.add_node(3);
    /// graph.add_dependency(first_node, second_node);
    /// graph.add_dependency(second_node, third_node);
    /// ```
    ///
    /// Also see [`graph_deps!`] macro. This macro provides a shorthand syntax for adding dependencies.
    pub fn add_dependency(&mut self, what: NodeId, depends_on_what: NodeId) {
        self.direct_deps
            .entry(what)
            .or_default()
            .insert(depends_on_what);
        self.invert_deps
            .entry(depends_on_what)
            .or_default()
            .insert(what);
        *self.in_degree.entry(depends_on_what).or_insert(0) += 1;
        *self.out_degree.entry(what).or_insert(0) += 1;
    }

    /// Validates and transforms the unconfigured graph into a ready-to-execute [`ConfiguredExecutionGraph`].
    ///
    /// This method validates the current state of the UnconfiguredExecutionGraph.
    /// It checks for cycles in the graph and performs topological sorting to ensure that nodes can be executed
    /// in a valid order. If the graph is valid (i.e., no cycles are found), it is transformed into a
    /// [`ConfiguredExecutionGraph`], which can be executed. If a cycle is detected, this method returns
    /// an error ([`UnconfiguredExecutionGraphError::GraphHasCycle`]).
    ///
    /// This method consumes the `UnconfiguredExecutionGraph`, meaning it cannot be used after a successful
    /// configuration unless the graph is recreated.
    pub fn configure(self) -> Result<ConfiguredExecutionGraph<T>> {
        debug!("Configuring executable node...");
        self.topological_sort()?;
        let nodes = self
            .nodes
            .into_iter()
            .enumerate()
            .map(|(idx, data)| {
                let id = NodeId(idx);
                (id, Node { id, data })
            })
            .collect::<DashMap<NodeId, _>>();
        let dependents = self
            .invert_deps
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();
        let incoming_edge_counts = self.out_degree;

        Ok(ConfiguredExecutionGraph {
            nodes: Arc::new(nodes),
            dependents,
            incoming_edge_counts,
        })
    }

    /// Performs topological sorting of the graph nodes using Kahn's algorithm.
    ///
    /// Returns topologically sorted set of node IDs if the graph is acyclic.
    /// Returns error if the graph contains cycles, the error contains detected cycle nodes.
    fn topological_sort(&self) -> Result<OrderSet<NodeId>> {
        // Clone the in-degree map to avoid modifying the original graph state
        let mut in_degree = self.in_degree.clone();
        // Queue for nodes with no incoming edges (in-degree = 0)
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        // Initialize queue with all nodes having zero in-degree
        in_degree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .for_each(|(id, _)| queue.push_back(*id));

        // Pre-allocate result vector for efficiency
        let mut result: Vec<NodeId> = Vec::with_capacity(self.nodes.len());
        let mut processed = 0; // Counter for processed nodes

        // Kahn's algorithm main loop
        while let Some(node_id) = queue.pop_front() {
            result.push(node_id);
            processed += 1;

            // If the node has outgoing edges, update in-degree of its neighbors
            if let Some(deps) = self.direct_deps.get(&node_id) {
                for neighbor in deps.iter() {
                    if let Some(entry) = in_degree.get_mut(neighbor) {
                        *entry -= 1; // Decrement neighbor's in-degree
                        // If neighbor's in-degree reaches zero, add to queue
                        if *entry == 0 {
                            queue.push_back(*neighbor);
                        }
                    }
                }
            }
        }

        // Cycle detection - if not all nodes were processed
        if processed != self.nodes.len() {
            // Collect all nodes remaining with non-zero in-degree (part of cycles)
            let cycle: Vec<NodeId> = in_degree
                .into_iter()
                // it.degree > 0
                .filter(|(_, degree)| *degree > 0)
                .map(|(id, _)| id)
                .collect();
            debug!("Found cycle during exec graph configuration: {cycle:?}");
            Err(UnconfiguredExecutionGraphError::GraphHasCycle { cycle })
        } else {
            // Convert Vec to OrderSet for deterministic iteration
            Ok(result.into_iter().collect())
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use crate::graph_deps;

    use super::*;

    #[test]
    fn configure_valid_adg__EXPECT__ok() {
        // Given
        let mut graph: UnconfiguredExecutionGraph<&str> = Default::default();
        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");
        graph_deps! { graph, a => b => c }

        // When
        let graph = graph.configure();

        // Then
        assert!(graph.is_ok())
    }

    #[test]
    fn configure_invalid_adg_with_cycle__EXPECT__err() {
        // Given
        let mut graph: UnconfiguredExecutionGraph<&str> = Default::default();
        let n0 = graph.add_node("0");
        let n1 = graph.add_node("1");
        let n2 = graph.add_node("2");
        graph_deps! { graph, n0 => n1 => n2 };
        graph_deps! { graph, n2 => n0 };

        // When
        let graph = graph.configure();

        // Then
        assert_eq!(
            UnconfiguredExecutionGraphError::GraphHasCycle {
                cycle: vec![n0, n1, n2]
            },
            graph.unwrap_err(),
        );
    }

    #[test]
    fn create_valid_adg__EXPECT__valid_in_degree_values() {
        // Given
        let mut graph: UnconfiguredExecutionGraph<&str> = Default::default();
        let n0 = graph.add_node("0");
        let n1 = graph.add_node("1");
        let n2 = graph.add_node("2");
        let n3 = graph.add_node("3");
        let n4 = graph.add_node("4");
        let expected_in_degree: OrderMap<NodeId, usize> = ordermap::ordermap! {
            n0 => 0,
            n1 => 1,
            n2 => 1,
            n3 => 1,
            n4 => 2,
        };

        // When
        // n0 -> n1 -> n2 -> n4
        //        '-> n3 -> '
        graph_deps! { graph, n0 => n1 => n2 => n4 };
        graph_deps! { graph, n1 => n3 => n4 };

        // Then
        assert_eq!(expected_in_degree, graph.in_degree);
    }

    #[test]
    fn create_valid_adg__EXPECT__valid_out_degree_values() {
        // Given
        let mut graph: UnconfiguredExecutionGraph<&str> = Default::default();
        let n0 = graph.add_node("0");
        let n1 = graph.add_node("1");
        let n2 = graph.add_node("2");
        let n3 = graph.add_node("3");
        let n4 = graph.add_node("4");
        let expected_out_degree: OrderMap<NodeId, usize> = ordermap::ordermap! {
            n0 => 1,
            n1 => 2,
            n2 => 1,
            n3 => 1,
            n4 => 0,
        };

        // When
        // n0 -> n1 -> n2 -> n4
        //        '-> n3 -> '
        graph_deps! { graph, n0 => n1 => n2 => n4 };
        graph_deps! { graph, n1 => n3 => n4 };

        // Then
        assert_eq!(expected_out_degree, graph.out_degree);
    }

    #[test]
    fn create_valid_adg__EXPECT__valid_invert_deps_values() {
        // Given
        let mut graph: UnconfiguredExecutionGraph<&str> = Default::default();
        let n0 = graph.add_node("0");
        let n1 = graph.add_node("1");
        let n2 = graph.add_node("2");
        let n3 = graph.add_node("3");
        let n4 = graph.add_node("4");
        let expected_invert_deps: HashMap<NodeId, HashSet<NodeId>> = {
            let mut map = HashMap::new();
            // nodes without invert deps does not have entries in the final map:
            // map.insert(n0, HashSet::new());
            map.insert(n1, [n0].into_iter().collect());
            map.insert(n2, [n1].into_iter().collect());
            map.insert(n3, [n1].into_iter().collect());
            map.insert(n4, [n2, n3].into_iter().collect());
            map
        };

        // When
        // n0 -> n1 -> n2 -> n4
        //        '-> n3 -->'
        graph_deps! { graph, n0 => n1 => n2 => n4 };
        graph_deps! { graph, n1 => n3 => n4 };

        // Then
        assert_eq!(expected_invert_deps, graph.invert_deps);
    }

    #[test]
    fn create_valid_adg__EXPECT__valid_direct_deps_values() {
        // Given
        let mut graph: UnconfiguredExecutionGraph<&str> = Default::default();
        let n0 = graph.add_node("0");
        let n1 = graph.add_node("1");
        let n2 = graph.add_node("2");
        let n3 = graph.add_node("3");
        let n4 = graph.add_node("4");
        let expected_direct_deps: HashMap<NodeId, HashSet<NodeId>> = {
            let mut map = HashMap::new();
            map.insert(n0, [n1].into_iter().collect());
            map.insert(n1, [n2, n3].into_iter().collect());
            map.insert(n2, [n4].into_iter().collect());
            map.insert(n3, [n4].into_iter().collect());
            // nodes without direct deps does not have entries in the final map:
            // map.insert(n4,  HashSet::new());
            map
        };

        // When
        // n0 -> n1 -> n2 -> n4
        //        '-> n3 -->'
        graph_deps! { graph, n0 => n1 => n2 => n4 };
        graph_deps! { graph, n1 => n3 => n4 };

        // Then
        assert_eq!(expected_direct_deps, graph.direct_deps);
    }
}
