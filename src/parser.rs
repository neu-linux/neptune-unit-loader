use petgraph::{Directed, algo::toposort, graph::Graph};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use crate::unit::{UnitFile, UnitType};

pub fn load_unit(path: &str) -> Result<UnitFile, String> {
  let path = Path::new(path);
  let Some(ext_str) = path.extension().and_then(|s| s.to_str()) else {
    return Err("Cannot load".to_string());
  };

  let Ok(ext_type) = UnitType::from_str(ext_str) else {
    return Err("Cannot load".to_string());
  };

  let contents = fs::read_to_string(&path)
    .map_err(|e| format!("Failed to read unit {}: {}", path.display(), e))?;

  let unit: UnitFile = toml::from_str(&contents)
    .map_err(|e| format!("Invalid unit in {}: {}", path.display(), e))?;

  if unit.validate().is_err() || unit.unit.unit_type != ext_type {
    return Err("Cannot load".to_string());
  }

  return Ok(unit);
}

fn load_units(path: &str) -> Result<Vec<UnitFile>, String> {
  let entries = fs::read_dir(path)
    .map_err(|e| format!("Failed to read unit directory: {}", e))?;

  let mut units = Vec::new();

  for entry in entries {
    let entry =
      entry.map_err(|e| format!("Invalid unit directory entry: {}", e))?;
    let path = entry.path();

    let Some(ext_str) = path.extension().and_then(|s| s.to_str()) else {
      continue;
    };

    let Ok(ext_type) = UnitType::from_str(ext_str) else {
      continue;
    };

    let contents = fs::read_to_string(&path)
      .map_err(|e| format!("Failed to read unit {}: {}", path.display(), e))?;

    let unit: UnitFile = toml::from_str(&contents)
      .map_err(|e| format!("Invalid unit in {}: {}", path.display(), e))?;

    if unit.validate().is_err() || unit.unit.unit_type != ext_type {
      continue;
    }

    units.push(unit);
  }

  Ok(units)
}

fn build_dependency_graph(units: &[UnitFile]) -> Graph<String, (), Directed> {
  let mut graph = Graph::new();
  let mut node_indices = HashMap::new();

  for unit in units {
    let name = unit.unit.unit_name.clone();
    let idx = graph.add_node(name.clone());
    node_indices.insert(name, idx);
  }

  for unit in units {
    let name = &unit.unit.unit_name;
    let Some(&from) = node_indices.get(name) else {
      continue;
    };

    let dependency = &unit.dependency;

    for dep in &dependency.needs_before {
      if let Some(&to) = node_indices.get(dep) {
        graph.add_edge(from, to, ());
      }
    }

    for dep in &dependency.needs_after {
      if let Some(&to) = node_indices.get(dep) {
        graph.add_edge(to, from, ());
      }
    }
  }

  graph
}

fn generate_unit_list(
  graph: &Graph<String, (), Directed>,
) -> Result<Vec<String>, String> {
  toposort(graph, None)
    .map_err(|cycle| {
      let node_id = cycle.node_id();
      let name = graph
        .node_weight(node_id)
        .map_or_else(|| "<unknown>".into(), ToString::to_string);
      format!("Cycle detected involving: {}", name)
    })
    .map(|sorted| {
      sorted
        .into_iter()
        .filter_map(|idx| graph.node_weight(idx).cloned())
        .collect()
    })
}

pub fn generate(units: &[UnitFile]) -> Result<Vec<String>, String> {
  let dependency_graph = build_dependency_graph(units);
  generate_unit_list(&dependency_graph)
}

pub fn load_and_generate(path: &str) -> Result<Vec<String>, String> {
  let loaded_units = load_units(path)?;
  generate(&loaded_units)
}
