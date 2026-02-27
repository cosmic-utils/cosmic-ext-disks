use std::fs;
use std::path::PathBuf;

use crate::cmd::{CommandOutcome, run};
use crate::errors::{Result, TestingError};
use crate::ledger::{self, SpecState};
use crate::spec::{artifacts_root, load_by_name};

#[derive(Debug, Clone)]
pub struct Plan {
    pub dry_run: bool,
    pub steps: Vec<String>,
}

fn image_paths(spec_name: &str) -> Result<Vec<PathBuf>> {
    let spec = load_by_name(spec_name)?;
    let root = artifacts_root(&spec);
    Ok(spec
        .images
        .iter()
        .map(|image| root.join(&image.file_name))
        .collect())
}

pub fn plan_create(spec_name: &str, dry_run: bool) -> Result<Plan> {
    let spec = load_by_name(spec_name)?;
    let root = artifacts_root(&spec);
    let mut steps = vec![format!("mkdir -p {}", root.display())];
    for image in &spec.images {
        let path = root.join(&image.file_name);
        steps.push(format!(
            "truncate -s {} {}",
            image.size_bytes,
            path.display()
        ));
    }

    Ok(Plan { dry_run, steps })
}

pub fn plan_prepare(spec_name: &str, dry_run: bool) -> Result<Plan> {
    let spec = load_by_name(spec_name)?;
    let paths = image_paths(spec_name)?;
    let mut steps = Vec::new();

    for path in paths {
        steps.push(format!(
            "parted -s {} mklabel {}",
            path.display(),
            spec.partition_table
        ));
        for partition in &spec.partitions {
            steps.push(format!(
                "parted -s {} mkpart {} {} {}",
                path.display(),
                partition.r#type,
                partition.start,
                partition.end
            ));
        }
    }

    Ok(Plan { dry_run, steps })
}

pub fn plan_attach(spec_name: &str, dry_run: bool) -> Result<Plan> {
    let paths = image_paths(spec_name)?;
    let steps = paths
        .iter()
        .map(|path| format!("losetup --find --show --partscan {}", path.display()))
        .collect();
    Ok(Plan { dry_run, steps })
}

pub fn plan_mount(spec_name: &str, dry_run: bool) -> Result<Plan> {
    let spec = load_by_name(spec_name)?;
    let state = ledger::load(spec_name)?;
    let mut steps = Vec::new();

    for (index, mount) in spec.mounts.iter().enumerate() {
        steps.push(format!("mkdir -p {}", mount.mount_point));
        let mapped = state
            .mapped_partitions
            .iter()
            .filter(|value| value.ends_with(&mount.partition_ref))
            .nth(index)
            .ok_or_else(|| TestingError::SpecInvalid {
                spec_name: spec_name.to_string(),
                reason: format!(
                    "partition ref not available in ledger: {}",
                    mount.partition_ref
                ),
            })?;
        let partition = spec
            .partitions
            .iter()
            .find(|partition| format!("p{}", partition.index) == mount.partition_ref);
        if let Some(partition) = partition
            && let Some(filesystem) = &partition.fs
        {
            steps.push(format!("mkfs.{} {}", filesystem, mapped));
        }
        steps.push(format!("mount {} {}", mapped, mount.mount_point));
    }

    Ok(Plan { dry_run, steps })
}

pub fn plan_unmount(spec_name: &str, dry_run: bool) -> Result<Plan> {
    let state = ledger::load(spec_name)?;
    let steps = state
        .mount_points
        .iter()
        .map(|mount| format!("umount {}", mount))
        .collect();
    Ok(Plan { dry_run, steps })
}

pub fn plan_detach(spec_name: &str, dry_run: bool) -> Result<Plan> {
    let state = ledger::load(spec_name)?;
    let steps = state
        .loop_devices
        .iter()
        .map(|loop_device| format!("losetup -d {}", loop_device))
        .collect();
    Ok(Plan { dry_run, steps })
}

pub fn plan_destroy(spec_name: &str, dry_run: bool) -> Result<Plan> {
    let paths = image_paths(spec_name)?;
    let steps = paths
        .iter()
        .map(|path| format!("rm -f {}", path.display()))
        .collect();
    Ok(Plan { dry_run, steps })
}

pub fn plan_cleanup(spec_name: &str, dry_run: bool) -> Result<Plan> {
    let mut all_steps = Vec::new();
    all_steps.extend(plan_unmount(spec_name, dry_run)?.steps);
    all_steps.extend(plan_detach(spec_name, dry_run)?.steps);
    all_steps.push(format!("rm -f {}", ledger::state_path(spec_name).display()));
    Ok(Plan {
        dry_run,
        steps: all_steps,
    })
}

pub fn plan_cleanup_all(dry_run: bool) -> Result<Plan> {
    let mut steps = Vec::new();
    let base_dir = ledger::base_dir();
    if !base_dir.exists() {
        return Ok(Plan { dry_run, steps });
    }

    let entries = fs::read_dir(&base_dir).map_err(|error| TestingError::LedgerIo {
        path: base_dir.clone(),
        reason: error.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|error| TestingError::LedgerIo {
            path: base_dir.clone(),
            reason: error.to_string(),
        })?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|value| value.to_str()) {
            let plan = plan_cleanup(stem, dry_run)?;
            steps.extend(plan.steps);
        }
    }

    Ok(Plan { dry_run, steps })
}

fn split_shell_command(step: &str) -> (String, Vec<String>) {
    let mut parts = step.split_whitespace();
    let command = parts.next().unwrap_or_default().to_string();
    let args = parts.map(std::string::ToString::to_string).collect();
    (command, args)
}

pub fn execute_plan(plan: &Plan) -> Result<Vec<CommandOutcome>> {
    let mut outcomes = Vec::new();
    for step in &plan.steps {
        let (command, args) = split_shell_command(step);
        if command == "mkdir" {
            if plan.dry_run {
                outcomes.push(CommandOutcome {
                    command: step.clone(),
                    stdout: String::new(),
                    stderr: String::new(),
                    executed: false,
                });
                continue;
            }
            if let Some(path) = args.last() {
                fs::create_dir_all(path).map_err(|error| TestingError::CommandFailed {
                    command: step.clone(),
                    stderr: error.to_string(),
                })?;
                outcomes.push(CommandOutcome {
                    command: step.clone(),
                    stdout: String::new(),
                    stderr: String::new(),
                    executed: true,
                });
                continue;
            }
        }

        outcomes.push(run(&command, &args, plan.dry_run)?);
    }
    Ok(outcomes)
}

pub fn record_attach_state(spec_name: &str, loops: &[String]) -> Result<()> {
    let spec = load_by_name(spec_name)?;
    let mut state = if ledger::exists(spec_name) {
        ledger::load(spec_name)?
    } else {
        SpecState::new(spec_name)
    };

    state.image_paths = spec
        .images
        .iter()
        .map(|image| {
            artifacts_root(&spec)
                .join(&image.file_name)
                .display()
                .to_string()
        })
        .collect();
    state.loop_devices = loops.to_vec();
    state.mapped_partitions = loops
        .iter()
        .flat_map(|loop_device| {
            spec.partitions
                .iter()
                .map(|partition| format!("{}p{}", loop_device, partition.index))
                .collect::<Vec<_>>()
        })
        .collect();

    ledger::save(&state)?;
    Ok(())
}

pub fn record_mount_state(spec_name: &str, mount_points: &[String]) -> Result<()> {
    let mut state = if ledger::exists(spec_name) {
        ledger::load(spec_name)?
    } else {
        SpecState::new(spec_name)
    };

    state.mount_points = mount_points.to_vec();
    ledger::save(&state)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::plan_create;

    #[test]
    fn create_plan_is_destructive_by_default() {
        let plan = plan_create("2disk", false).unwrap();
        assert!(!plan.dry_run);
        assert!(!plan.steps.is_empty());
    }
}
