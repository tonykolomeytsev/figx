//! # Utils for `lib_graph_exec` module

/// A simple macro for declaring a chain of dependencies in a graph.
/// 
/// This macro provides a shorthand syntax for adding dependencies 
/// between nodes in a graph. Instead of calling `.add_dependency(a, b)` 
/// repeatedly, you can use a chain-like syntax to express multiple 
/// dependencies in a single line. The macro expands into successive calls 
/// to the graph's `add_dependency` method.
/// 
/// # Examples
/// 
/// Adding simple dependency chain:
/// 
/// ```
/// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
/// # use lib_graph_exec::graph_deps;
/// # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
/// # let a = graph.add_node(1);
/// # let b = graph.add_node(2);
/// # let c = graph.add_node(3);
/// // Arrow '=>' direction shows dependency direction (what => depends_on_what)
/// graph_deps! { graph, a => b => c };
/// ```
/// 
/// Adding multiple dependency chains:
/// 
/// ```text
/// a -- b -- c -- d
///  \            /
///   \          /
///    e ------ f
/// ```
/// 
/// ```
/// # use lib_graph_exec::unconfigured::UnconfiguredExecutionGraph;
/// # use lib_graph_exec::graph_deps;
/// # let mut graph: UnconfiguredExecutionGraph<i32> = Default::default();
/// # let a = graph.add_node(1);
/// # let b = graph.add_node(2);
/// # let c = graph.add_node(3);
/// # let d = graph.add_node(4);
/// # let e = graph.add_node(5);
/// # let f = graph.add_node(6);
/// // Arrow '=>' direction shows dependency direction (what => depends_on_what)
/// graph_deps! { graph, a => b => c => d };
/// graph_deps! { graph, a => e => f => d };
/// ```
/// 
#[macro_export]
macro_rules! graph_deps {
    // Базовый случай: два элемента (a => b)
    ($graph:expr, $a:expr => $b:expr) => {
        $graph.add_dependency($a, $b);
    };
    
    // Рекурсивный случай: разбираем цепочку (a => b => ...)
    ($graph:expr, $a:expr => $b:expr => $($rest:tt)*) => {
        $graph.add_dependency($a, $b);
        graph_deps!($graph, $b => $($rest)*);
    };
}
