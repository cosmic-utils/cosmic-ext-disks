use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::{Result, TestingError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabSpec {
    pub name: String,
    pub artifacts_root: Option<String>,
    pub images: Vec<ImageSpec>,
    pub partition_table: String,
    pub partitions: Vec<PartitionSpec>,
    pub mounts: Vec<MountSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSpec {
    pub file_name: String,
    pub size_bytes: u64,
    pub loop_device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionSpec {
    pub index: u32,
    pub start: String,
    pub end: String,
    pub r#type: String,
    pub fs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountSpec {
    pub partition_ref: String,
    pub mount_point: String,
}

pub fn workspace_root() -> PathBuf {
    if let Ok(value) = std::env::var("STORAGE_TESTING_WORKSPACE_ROOT") {
        return PathBuf::from(value);
    }

    if let Ok(current_dir) = std::env::current_dir()
        && current_dir.join("resources/lab-specs").exists()
    {
        return current_dir;
    }

    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    if manifest_root.join("resources/lab-specs").exists() {
        return manifest_root;
    }

    PathBuf::from(".")
}

pub fn specs_root() -> PathBuf {
    workspace_root().join("resources/lab-specs")
}

pub fn spec_path_for_name(spec_name: &str) -> PathBuf {
    specs_root().join(format!("{}.toml", spec_name))
}

pub fn load_by_name(spec_name: &str) -> Result<LabSpec> {
    let path = spec_path_for_name(spec_name);
    if !path.exists() {
        return Err(TestingError::SpecNotFound {
            spec_name: spec_name.to_string(),
        });
    }

    let raw = fs::read_to_string(&path).map_err(|error| TestingError::SpecInvalid {
        spec_name: spec_name.to_string(),
        reason: error.to_string(),
    })?;

    let spec: LabSpec = toml::from_str(&raw).map_err(|error| TestingError::SpecInvalid {
        spec_name: spec_name.to_string(),
        reason: error.to_string(),
    })?;

    validate(&spec)?;
    Ok(spec)
}

pub fn validate(spec: &LabSpec) -> Result<()> {
    if spec.name.is_empty() {
        return Err(TestingError::SpecInvalid {
            spec_name: "<unknown>".to_string(),
            reason: "name must not be empty".to_string(),
        });
    }

    if spec.images.is_empty() {
        return Err(TestingError::SpecInvalid {
            spec_name: spec.name.clone(),
            reason: "images must not be empty".to_string(),
        });
    }

    if spec.partition_table != "gpt" && spec.partition_table != "dos" {
        return Err(TestingError::SpecInvalid {
            spec_name: spec.name.clone(),
            reason: "partition_table must be 'gpt' or 'dos'".to_string(),
        });
    }

    if spec.partitions.is_empty() {
        return Err(TestingError::SpecInvalid {
            spec_name: spec.name.clone(),
            reason: "partitions must not be empty".to_string(),
        });
    }

    Ok(())
}

pub fn artifacts_root(spec: &LabSpec) -> PathBuf {
    match &spec.artifacts_root {
        Some(value) => workspace_root().join(value),
        None => workspace_root().join(format!("target/storage-testing/images/{}", spec.name)),
    }
}

#[cfg(test)]
mod tests {
    use super::load_by_name;

    #[test]
    fn resolves_spec_name_without_extension() {
        let spec = load_by_name("2disk").unwrap();
        assert_eq!(spec.name, "2disk");
    }
}
