use std::collections::HashMap;

use petgraph::algo::toposort;
use petgraph::{Directed, Graph};

use crate::parser::loader::load_units;
use crate::unit::errors::{GraphBuildError, UnitLoadError};
use crate::unit::types::UnitFile;

pub fn generate_unit_list(units: &[UnitFile]) -> Result<Vec<UnitFile>, GraphBuildError> {
    let mut graph = Graph::<usize, (), Directed>::new();
    let mut idx_map = HashMap::new();

    for (i, unit) in units.iter().enumerate() {
        let node_idx = graph.add_node(i);
        idx_map.insert(unit.unit.unit_name.clone(), node_idx);
    }

    for unit in units {
        let from = *idx_map.get(&unit.unit.unit_name).unwrap();

        for dep in &unit.dependency.needs_before {
            if let Some(&to) = idx_map.get(dep) {
                graph.add_edge(from, to, ());
            } else {
                return Err(GraphBuildError::LoadError(UnitLoadError::MissingDependency(
                    unit.unit.unit_name.clone(),
                    dep.clone(),
                )));
            }
        }
        for dep in &unit.dependency.needs_after {
            if let Some(&to) = idx_map.get(dep) {
                graph.add_edge(to, from, ());
            } else {
                return Err(GraphBuildError::LoadError(UnitLoadError::MissingDependency(
                    unit.unit.unit_name.clone(),
                    dep.clone(),
                )));
            }
        }
    }

    let sorted = toposort(&graph, None).map_err(|cycle| {
        let node = cycle.node_id();
        let name = units[graph[node]].unit.unit_name.clone();
        GraphBuildError::DependencyCycle(name)
    })?;

    let ordered_units = sorted.into_iter().map(|node_idx| units[graph[node_idx]].clone()).collect();

    Ok(ordered_units)
}

pub fn generate(units: &[UnitFile]) -> Result<Vec<UnitFile>, GraphBuildError> {
    generate_unit_list(units)
}

pub fn load_and_generate(path: &str) -> Result<Vec<UnitFile>, GraphBuildError> {
    let loaded_units = load_units(path)?;
    generate(&loaded_units)
}
