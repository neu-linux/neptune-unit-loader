use std::{fs, path::Path, str::FromStr};

use log::warn as logwarn;

use crate::unit::errors::UnitLoadError;
use crate::unit::types::{UnitFile, UnitType};

pub fn load_unit(path: &str) -> Result<UnitFile, UnitLoadError> {
    let path = Path::new(path);
    let ext_str =
        path.extension().and_then(|s| s.to_str()).ok_or(UnitLoadError::InvalidExtension)?;

    let ext_type = UnitType::from_str(ext_str)
        .map_err(|_| UnitLoadError::UnsupportedUnitType(ext_str.to_string()))?;

    let contents =
        fs::read_to_string(path).map_err(|e| UnitLoadError::ReadError(path.to_path_buf(), e))?;

    let unit: UnitFile =
        toml::from_str(&contents).map_err(|e| UnitLoadError::ParseError(path.to_path_buf(), e))?;

    if unit.validate().is_err() || unit.unit.unit_type != ext_type {
        return Err(UnitLoadError::ValidationError(path.to_path_buf()));
    }

    Ok(unit)
}

pub fn load_units(dir_path: &str) -> Result<Vec<UnitFile>, UnitLoadError> {
    let dir = Path::new(dir_path);
    let entries =
        fs::read_dir(dir).map_err(|e| UnitLoadError::ReadDirError(dir.to_path_buf(), e))?;

    let mut units = Vec::new();

    for entry_result in entries {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                return Err(UnitLoadError::DirEntryError(dir.to_path_buf(), e));
            }
        };

        let pathbuf = entry.path();
        let path = match pathbuf.to_str() {
            Some(p) => p,
            None => {
                logwarn!("Skipping entry with invalid UTF-8 path: {:?}", pathbuf);
                continue;
            }
        };

        let display_path = pathbuf.display();

        match load_unit(path) {
            Ok(unit) => {
                units.push(unit);
            }
            Err(err) => {
                logwarn!("Skipping {} due to error: {}", display_path, err);
            }
        }
    }

    Ok(units)
}
