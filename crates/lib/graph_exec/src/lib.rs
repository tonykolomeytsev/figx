//! # `lib_graph_exec` — Action Graph Execution Library
//!
//! A library for constructing and executing Action Dependency Graphs (ADGs)
//! in a domain-agnostic manner.
//!
//! This library enables the construction of ADGs, where arbitrary data can be
//! transferred into the graph with full ownership and retrieved back with
//! ownership preserved during the execution of actions. The library performs
//! automatic validation of the dependency graph and determines the correct
//! execution order of nodes based on their dependencies. Nodes are executed in
//! parallel using a Rayon thread pool; no node will start until all of its
//! dependencies have successfully completed.
//!
//! Also see [`graph_deps!`] macro.

#![warn(missing_docs)]

pub mod action;
pub mod configured;
pub mod unconfigured;
pub mod util;

/// A identifier type for nodes in a graph.
///
/// NodeId is a simple, strongly-typed wrapper around a [`usize`] value, used to uniquely
/// identify nodes within a graph. It is intended to provide type safety when referencing
/// nodes, while remaining efficient to copy and hash. The type derives common traits,
/// making it suitable for use as a key in hash maps or sets.
///
/// NodeId values cannot be constructed directly by users. They are only returned by the graph
/// when a node is added via an appropriate method ([`unconfigured::UnconfiguredExecutionGraph::add_node`]).
/// This restriction preserves the internal consistency and encapsulation of the Graph aggregate,
/// ensuring that all node identifiers are valid and meaningful within the graph’s structure.
///
/// # Example
///
/// Graph nodes with same values have different node IDs:
///
/// ```
/// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
/// # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
/// let first_node = graph.add_node(42);
/// let second_node = graph.add_node(42);
/// assert_ne!(first_node, second_node);
/// ```
///
/// Using as a key in a hash map:
///
/// ```
/// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
/// # use std::collections::HashMap;
/// # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
/// let first_node = graph.add_node(42);
/// let mut map = HashMap::new();
/// map.insert(first_node, "this is first node meta");
/// assert_eq!(Some(&"this is first node meta"), map.get(&first_node));
/// ```
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct NodeId(usize);
