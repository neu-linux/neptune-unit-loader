use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use log::warn as logwarn;
use petgraph::Directed;
use petgraph::algo::toposort;
use petgraph::graph::Graph;
use thiserror::Error;

use crate::unit::{UnitFile, UnitType};

#[derive(Debug, Error,)]
pub enum UnitLoadError {
    #[error("Missing or invalid file extension")]
    InvalidExtension,

    #[error("Unsupported unit type extension: {0}")]
    UnsupportedUnitType(String,),

    #[error("Failed to read unit file {0}: {1}")]
    ReadError(PathBuf, #[source] std::io::Error,),

    #[error("Invalid unit format in file {0}: {1}")]
    ParseError(PathBuf, #[source] toml::de::Error,),

    #[error("Validation failed or unit type mismatch in {0}")]
    ValidationError(PathBuf,),

    #[error("Failed to read unit directory {0}: {1}")]
    ReadDirError(PathBuf, #[source] std::io::Error,),

    #[error("Invalid unit file entry in directory {0}: {1}")]
    DirEntryError(PathBuf, #[source] std::io::Error,),
}

#[derive(Debug, Error,)]
pub enum GraphBuildError {
    #[error(transparent)]
    LoadError(#[from] UnitLoadError,),

    #[error("Cycle detected involving: {0}")]
    DependencyCycle(String,),
}

pub fn load_unit(path: &str,) -> Result<UnitFile, UnitLoadError,> {
    let path = Path::new(path,);
    let ext_str =
        path.extension().and_then(|s| s.to_str(),).ok_or(UnitLoadError::InvalidExtension,)?;

    let ext_type = UnitType::from_str(ext_str,)
        .map_err(|_| UnitLoadError::UnsupportedUnitType(ext_str.to_string(),),)?;

    let contents =
        fs::read_to_string(path,).map_err(|e| UnitLoadError::ReadError(path.to_path_buf(), e,),)?;

    let unit: UnitFile = toml::from_str(&contents,)
        .map_err(|e| UnitLoadError::ParseError(path.to_path_buf(), e,),)?;

    if unit.validate().is_err() || unit.unit.unit_type != ext_type {
        return Err(UnitLoadError::ValidationError(path.to_path_buf(),),);
    }

    Ok(unit,)
}

pub fn load_units(dir_path: &str,) -> Result<Vec<UnitFile,>, UnitLoadError,> {
    let dir = Path::new(dir_path,);
    let entries =
        fs::read_dir(dir,).map_err(|e| UnitLoadError::ReadDirError(dir.to_path_buf(), e,),)?;

    let mut units = Vec::new();

    for entry_result in entries {
        let entry = match entry_result {
            Ok(e,) => e,
            Err(e,) => {
                return Err(UnitLoadError::DirEntryError(dir.to_path_buf(), e,),);
            }
        };

        let pathbuf = entry.path();
        let Some(path,) = pathbuf.to_str() else {
            continue;
        };

        let display_path = pathbuf.display();

        match load_unit(path,) {
            Ok(unit,) => {
                units.push(unit,);
            }
            Err(err,) => {
                logwarn!("Warning: skipping {} due to error: {}", display_path, err);
            }
        }
    }

    Ok(units,)
}

fn build_dependency_graph(units: &[UnitFile],) -> Graph<String, (), Directed,> {
    let mut graph = Graph::new();
    let mut node_indices = HashMap::new();

    for unit in units {
        let name = unit.unit.unit_name.clone();
        let idx = graph.add_node(name.clone(),);
        node_indices.insert(name, idx,);
    }

    for unit in units {
        let name = &unit.unit.unit_name;
        let Some(&from,) = node_indices.get(name,) else {
            continue;
        };

        let dependency = &unit.dependency;

        for dep in &dependency.needs_before {
            if let Some(&to,) = node_indices.get(dep,) {
                graph.add_edge(from, to, (),);
            } else {
                logwarn!("{} depends on missing unit '{}'", name, dep)
            }
        }

        for dep in &dependency.needs_after {
            if let Some(&to,) = node_indices.get(dep,) {
                graph.add_edge(to, from, (),);
            } else {
                logwarn!("{} depends on missing unit '{}'", name, dep)
            }
        }
    }

    graph
}

fn generate_unit_list(
    graph: &Graph<String, (), Directed,>,
) -> Result<Vec<String,>, GraphBuildError,> {
    let sorted_nodes = toposort(graph, None,).map_err(|cycle| {
        let node_id = cycle.node_id();
        let node_name = graph
            .node_weight(node_id,)
            .map(ToString::to_string,)
            .unwrap_or_else(|| "<unknown>".into(),);
        GraphBuildError::DependencyCycle(node_name,)
    },)?;

    let unit_names =
        sorted_nodes.into_iter().filter_map(|idx| graph.node_weight(idx,).cloned(),).collect();

    Ok(unit_names,)
}

pub fn generate(units: &[UnitFile],) -> Result<Vec<String,>, GraphBuildError,> {
    let dependency_graph = build_dependency_graph(units,);
    generate_unit_list(&dependency_graph,)
}

pub fn load_and_generate(path: &str,) -> Result<Vec<String,>, GraphBuildError,> {
    let loaded_units = load_units(path,)?;
    generate(&loaded_units,)
}
