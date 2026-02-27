use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::errors::{Result, TestingError};
use crate::spec::workspace_root;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndex {
    pub run_label: String,
    pub run_id: String,
}

pub fn artifacts_root() -> PathBuf {
    workspace_root().join("target/storage-testing/artifacts")
}

pub fn run_dir(run_label: &str) -> Result<PathBuf> {
    let run_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs()
        .to_string();

    let dir = artifacts_root().join(format!("{}-{}", run_label, run_id));
    fs::create_dir_all(&dir).map_err(|error| TestingError::LedgerIo {
        path: dir.clone(),
        reason: error.to_string(),
    })?;

    let index = RunIndex {
        run_label: run_label.to_string(),
        run_id,
    };
    let index_path = dir.join("index.json");
    let content = serde_json::to_string_pretty(&index).map_err(|error| TestingError::LedgerIo {
        path: index_path.clone(),
        reason: error.to_string(),
    })?;

    fs::write(&index_path, content).map_err(|error| TestingError::LedgerIo {
        path: index_path,
        reason: error.to_string(),
    })?;

    for log_name in ["commands.log", "service.log", "test.log"] {
        let log_path = dir.join(log_name);
        fs::write(&log_path, "").map_err(|error| TestingError::LedgerIo {
            path: log_path,
            reason: error.to_string(),
        })?;
    }

    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::run_dir;

    #[test]
    fn writes_artifact_index_on_run() {
        let dir = run_dir("smoke").unwrap();
        let idx = dir.join("index.json");
        assert!(idx.exists());
        assert!(dir.join("commands.log").exists());
        assert!(dir.join("service.log").exists());
        assert!(dir.join("test.log").exists());
    }
}
