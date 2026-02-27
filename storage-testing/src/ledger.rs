use std::fs;
use std::path::{Path, PathBuf};

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

fn state_path_in(base: &Path, spec_name: &str) -> PathBuf {
    base.join(format!("{}.json", spec_name))
}

pub fn state_path(spec_name: &str) -> PathBuf {
    state_path_in(&base_dir(), spec_name)
}

pub fn exists(spec_name: &str) -> bool {
    state_path(spec_name).exists()
}

pub fn load(spec_name: &str) -> Result<SpecState> {
    load_from_dir(&base_dir(), spec_name)
}

fn load_from_dir(dir: &Path, spec_name: &str) -> Result<SpecState> {
    let path = state_path_in(dir, spec_name);
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
    save_to_dir(&base_dir(), state)
}

fn save_to_dir(dir: &Path, state: &SpecState) -> Result<PathBuf> {
    let dir = dir.to_path_buf();
    fs::create_dir_all(&dir).map_err(|error| TestingError::LedgerIo {
        path: dir.clone(),
        reason: error.to_string(),
    })?;

    let path = state_path_in(&dir, &state.spec_name);
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
    use super::{SpecState, load_from_dir, save_to_dir};
    use std::fs;

    #[test]
    fn persists_and_loads_spec_state() {
        let test_root = std::env::temp_dir().join(format!(
            "storage-testing-ledger-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&test_root).unwrap();

        let state = SpecState::new("2disk");
        let path = save_to_dir(&test_root, &state).unwrap();
        let loaded = load_from_dir(&test_root, "2disk").unwrap();
        assert_eq!(loaded.spec_name, "2disk");
        assert!(path.exists());

        let _ = fs::remove_dir_all(&test_root);
    }
}
