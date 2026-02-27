use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::errors::{Result, TestingError};
use crate::spec::workspace_root;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecState {
    pub spec_name: String,
    pub image_paths: Vec<String>,
    pub loop_devices: Vec<String>,
    pub mapped_partitions: Vec<String>,
    pub mount_points: Vec<String>,
    pub updated_at: String,
}

impl SpecState {
    pub fn new(spec_name: &str) -> Self {
        Self {
            spec_name: spec_name.to_string(),
            image_paths: Vec::new(),
            loop_devices: Vec::new(),
            mapped_partitions: Vec::new(),
            mount_points: Vec::new(),
            updated_at: chrono_like_timestamp(),
        }
    }
}

fn chrono_like_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0));
    format!("{}", duration.as_secs())
}

pub fn base_dir() -> PathBuf {
    workspace_root().join("target/storage-testing/lab-state")
}

pub fn state_path(spec_name: &str) -> PathBuf {
    base_dir().join(format!("{}.json", spec_name))
}

pub fn exists(spec_name: &str) -> bool {
    state_path(spec_name).exists()
}

pub fn load(spec_name: &str) -> Result<SpecState> {
    let path = state_path(spec_name);
    let raw = fs::read_to_string(&path).map_err(|error| TestingError::LedgerIo {
        path: path.clone(),
        reason: error.to_string(),
    })?;

    serde_json::from_str(&raw).map_err(|error| TestingError::LedgerIo {
        path,
        reason: error.to_string(),
    })
}

pub fn save(state: &SpecState) -> Result<PathBuf> {
    let dir = base_dir();
    fs::create_dir_all(&dir).map_err(|error| TestingError::LedgerIo {
        path: dir.clone(),
        reason: error.to_string(),
    })?;

    let path = state_path(&state.spec_name);
    let content = serde_json::to_string_pretty(state).map_err(|error| TestingError::LedgerIo {
        path: path.clone(),
        reason: error.to_string(),
    })?;

    fs::write(&path, content).map_err(|error| TestingError::LedgerIo {
        path: path.clone(),
        reason: error.to_string(),
    })?;

    Ok(path)
}

pub fn remove(spec_name: &str) -> Result<()> {
    let path = state_path(spec_name);
    if path.exists() {
        fs::remove_file(&path).map_err(|error| TestingError::LedgerIo {
            path,
            reason: error.to_string(),
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{SpecState, load, save};

    #[test]
    fn persists_and_loads_spec_state() {
        let state = SpecState::new("2disk");
        let path = save(&state).unwrap();
        let loaded = load("2disk").unwrap();
        assert_eq!(loaded.spec_name, "2disk");
        assert!(path.exists());
    }
}
