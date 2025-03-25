use lib_graph_exec::{NodeId, action::Action, configured::ConfiguredExecutionGraph};
use lib_label::LabelPattern;
use owo_colors::OwoColorize;
use std::collections::HashSet;
mod error;
pub use error::*;
use phase_evaluation::builder::EvalBuilder;

pub struct FeatureAQueryOptions {
    pub pattern: Vec<String>,
}

#[derive(Default)]
struct Node {
    name: String,
    children: Vec<Node>,
    params: Vec<(String, String)>,
}

pub fn query(opts: FeatureAQueryOptions) -> Result<()> {
    let pattern = LabelPattern::try_from(opts.pattern)?;
    let ws = phase_loading::load_workspace(pattern)?;
    let graph = EvalBuilder::from_workspace(&ws)
        .fetch_remotes(false)
        .fetch_resources()
        .transform_resources()
        .materialize_resources()
        .wrap_to_diagnostics()
        .build()?
        .into_inner();

    let nodes = to_diagnostics_nodes(&graph);
    for node in nodes {
        println!("{node}");
    }

    Ok(())
}

fn to_diagnostics_nodes<T, E, S>(
    graph: &ConfiguredExecutionGraph<Box<dyn Action<T, E, S>>>,
) -> Vec<Node>
where
    T: Send + Sync,
    E: Send + Sync,
    S: Send + Sync,
{
    // Find roots (or leafs idk) without any dependents
    let mut root_node_ids = HashSet::new();
    for node in graph.nodes.iter() {
        if let None = graph.dependents.get(&node.id) {
            root_node_ids.insert(node.id);
        }
    }

    let mut nodes = Vec::new();

    for entry in graph.nodes.iter() {
        let node_id = entry.key();
        if root_node_ids.contains(node_id) {
            let mut visited = HashSet::new();
            nodes.push(build_node_tree(graph, node_id, &mut visited));
        }
    }
    nodes
}

fn build_node_tree<T, E, S>(
    graph: &ConfiguredExecutionGraph<Box<dyn Action<T, E, S>>>,
    node_id: &NodeId,
    visited: &mut HashSet<NodeId>,
) -> Node
where
    T: Send + Sync,
    E: Send + Sync,
    S: Send + Sync,
{
    let mut node = Node::default();

    if let Some(entry) = graph.nodes.get(node_id) {
        let node_data = entry.value();
        let diagnostics = node_data.data.diagnostics_info();
        node.name = diagnostics.name;
        node.params = diagnostics.params;

        if visited.contains(node_id) {
            node.name = format!("{} (*)", node.name);
            return node;
        } else {
            visited.insert(*node_id);
        }

        // Find all nodes that this node depends on (inverse of dependents)
        let dependencies: Vec<NodeId> = graph
            .dependents
            .iter()
            .filter(|(_, dependents)| dependents.contains(node_id))
            .map(|(id, _)| id.clone())
            .collect();

        for dependency_id in dependencies {
            let parent_node = build_node_tree(graph, &dependency_id, visited);
            node.children.push(parent_node);
        }
    }
    node
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_tree(f, "")
    }
}

impl Node {
    fn fmt_tree(&self, f: &mut std::fmt::Formatter<'_>, prefix: &str) -> std::fmt::Result {
        // Выводим текущий узел
        writeln!(f, "{}", self.name.bold())?;
        for (param_key, param_value) in &self.params {
            let param_key = format!("{param_key}: ");
            writeln!(
                f,
                "{prefix}{} {}{}",
                "┆".bright_black(),
                param_key.blue(),
                param_value
            )?;
        }

        // Обрабатываем всех детей кроме последнего
        let middle_children = self.children.len().saturating_sub(1);
        for child in self.children.iter().take(middle_children) {
            // Префикс для текущего узла
            write!(f, "{prefix}{corner} ", corner = "├──".bright_black())?;
            // Префикс для детей текущего узла
            let new_prefix = format!("{prefix}{border}   ", border = "│".bright_black());
            child.fmt_tree(f, &new_prefix)?;
        }

        // Обрабатываем последнего ребенка (если есть)
        if let Some(last_child) = self.children.last() {
            // Префикс для последнего узла
            write!(f, "{prefix}{corner} ", corner = "╰──".bright_black())?;
            // Префикс для детей последнего узла (пробелы вместо │)
            let new_prefix = format!("{prefix}    ");
            last_child.fmt_tree(f, &new_prefix)?;
        }

        Ok(())
    }
}
