use std::collections::HashMap;

use serde::Deserialize;
use strum_macros::{Display, EnumString};

#[derive(Debug, Deserialize, PartialEq, Eq, EnumString, Display, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum UnitType {
    Service,
    Target,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnitSection {
    #[serde(rename = "name")]
    pub unit_name: String,
    pub description: Option<String>,

    #[serde(rename = "type")]
    pub unit_type: UnitType,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TodoSection {
    pub path: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServiceSection {
    #[serde(rename = "restart")]
    pub command_on_restart: Option<String>,

    #[serde(rename = "stop")]
    pub command_on_stop: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TargetSection {
    #[serde(rename = "once")]
    pub is_runnable_once: bool,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct DependencySection {
    #[serde(rename = "before", default)]
    pub needs_before: Vec<String>,

    #[serde(rename = "after", default)]
    pub needs_after: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UnitFile {
    pub unit: UnitSection,
    pub todo: TodoSection,

    #[serde(default)]
    pub service: Option<ServiceSection>,

    #[serde(default)]
    pub target: Option<TargetSection>,

    #[serde(default)]
    pub dependency: DependencySection,
}

impl UnitFile {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.unit.unit_name.trim().is_empty() {
            errors.push("Unit name cannot be empty".to_string());
        }

        if self.todo.path.trim().is_empty() {
            errors.push("Todo path cannot be empty".to_string());
        }

        match self.unit.unit_type {
            UnitType::Service if self.service.is_none() => {
                errors.push("Service unit requires [service] section".to_string());
            }
            UnitType::Target if self.target.is_none() => {
                errors.push("Target unit requires [target] section".to_string());
            }
            _ => {}
        }

        for dep in self.dependency.needs_before.iter().chain(&self.dependency.needs_after) {
            if dep.trim().is_empty() {
                errors.push("Dependency name cannot be empty".to_string());
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
