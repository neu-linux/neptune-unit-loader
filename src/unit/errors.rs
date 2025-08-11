use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnitLoadError {
    #[error("Missing or invalid file extension")]
    InvalidExtension,

    #[error("Unsupported unit type extension: {0}")]
    UnsupportedUnitType(String),

    #[error("Failed to read unit file {0}: {1}")]
    ReadError(PathBuf, #[source] std::io::Error),

    #[error("Invalid unit format in file {0}: {1}")]
    ParseError(PathBuf, #[source] toml::de::Error),

    #[error("Validation failed or unit type mismatch in {0}")]
    ValidationError(PathBuf),

    #[error("Failed to read unit directory {0}: {1}")]
    ReadDirError(PathBuf, #[source] std::io::Error),

    #[error("Invalid unit file entry in directory {0}: {1}")]
    DirEntryError(PathBuf, #[source] std::io::Error),

    #[error("\"{0}\" depends on missing unit \"{1}\"")]
    MissingDependency(String, String),
}

#[derive(Debug, Error)]
pub enum GraphBuildError {
    #[error(transparent)]
    LoadError(#[from] UnitLoadError),

    #[error("Cycle detected involving: {0}")]
    DependencyCycle(String),
}
