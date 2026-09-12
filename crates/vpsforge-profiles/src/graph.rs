use std::collections::{HashMap, HashSet};

use petgraph::algo::toposort;
use petgraph::graph::DiGraph;

use crate::model::{Component, Profile};

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub profile: String,
    pub section: String,
    pub component: Component,
}

/// Build a dependency graph across one or more profiles and return
/// components in install order. Edges mean "must install before".
pub fn ordered_components(profiles: &[Profile]) -> Result<Vec<GraphNode>, String> {
    let mut graph = DiGraph::<GraphNode, ()>::new();
    let mut section_nodes: HashMap<(String, String), Vec<petgraph::graph::NodeIndex>> =
        HashMap::new();

    for profile in profiles {
        for section in &profile.sections {
            let mut idxs = Vec::new();
            for component in &section.components {
                let idx = graph.add_node(GraphNode {
                    profile: profile.id.clone(),
                    section: section.id.clone(),
                    component: component.clone(),
                });
                idxs.push(idx);
            }
            section_nodes.insert((profile.id.clone(), section.id.clone()), idxs);
        }
    }

    for profile in profiles {
        for section in &profile.sections {
            let Some(targets) = section_nodes.get(&(profile.id.clone(), section.id.clone())) else {
                continue;
            };
            for dep in &section.depends_on {
                if let Some(sources) = section_nodes.get(&(profile.id.clone(), dep.clone())) {
                    for src in sources {
                        for dst in targets {
                            graph.add_edge(*src, *dst, ());
                        }
                    }
                }
            }
        }
    }

    let order = toposort(&graph, None).map_err(|_| {
        "circular dependency detected between profile sections".to_string()
    })?;

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for idx in order {
        let node = graph[idx].clone();
        if seen.insert(node.component.id.clone()) {
            out.push(node);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_embedded;

    #[test]
    fn ai_orders_python_before_ai_tools() {
        let profiles = load_embedded().unwrap();
        let ai = profiles.into_iter().find(|p| p.id == "ai").unwrap();
        let ordered = ordered_components(&[ai]).unwrap();
        let ids: Vec<_> = ordered.iter().map(|n| n.component.id.as_str()).collect();
        let python = ids.iter().position(|id| *id == "python3").unwrap();
        let ollama = ids.iter().position(|id| *id == "ollama").unwrap();
        assert!(python < ollama);
    }
}
