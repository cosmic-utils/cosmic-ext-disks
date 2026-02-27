use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use crate::cmd::{run, CommandOutcome};
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
    let spec = load_by_name(spec_name)?;
    let root = artifacts_root(&spec);
    let mut steps = Vec::new();

    for image in &spec.images {
        let path = root.join(&image.file_name);
        steps.push(format!("losetup --find --show --partscan {}", path.display()));
    }

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
    let steps: Vec<String> = state
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
        let is_fixed_loop_attach = command == "losetup"
            && args.len() >= 4
            && args[0] == "--show"
            && args[1] == "--partscan"
            && args[2].starts_with("/dev/loop");
        let is_dynamic_loop_attach = command == "losetup"
            && args.len() >= 3
            && args[0] == "--find"
            && args[1] == "--show"
            && args[2] == "--partscan";

        if command == "losetup"
            && args.len() >= 4
            && args[0] == "--show"
            && args[1] == "--partscan"
            && args[2].starts_with("/dev/loop")
        {
            let _ = ensure_loop_device_available(&args[2], plan.dry_run);
        }

        if command.starts_with("mkfs.")
            && let Some(device) = args.first()
        {
            ensure_partition_device_available(device, plan.dry_run)?;
        }

        if command == "mount"
            && let Some(device) = args.first()
        {
            ensure_partition_device_available(device, plan.dry_run)?;
        }

        if command == "umount"
            && let Some(mount_path) = args.first()
        {
            if !Path::new(mount_path).exists() || !is_mountpoint(mount_path) {
                outcomes.push(CommandOutcome {
                    command: step.clone(),
                    stdout: String::new(),
                    stderr: String::new(),
                    executed: false,
                });
                continue;
            }
        }

        if command == "losetup"
            && args.len() >= 2
            && args[0] == "-d"
            && !is_loop_attached(&args[1])
        {
            outcomes.push(CommandOutcome {
                command: step.clone(),
                stdout: String::new(),
                stderr: String::new(),
                executed: false,
            });
            continue;
        }

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

        if is_fixed_loop_attach {
            let mut last_error = None;
            for _ in 0..6 {
                match run(&command, &args, plan.dry_run) {
                    Ok(outcome) => {
                        outcomes.push(outcome);
                        last_error = None;
                        break;
                    }
                    Err(TestingError::CommandFailed { command, stderr })
                        if stderr.contains("Device or resource busy") =>
                    {
                        last_error = Some(TestingError::CommandFailed { command, stderr });
                        let _ = ensure_loop_device_available(&args[2], plan.dry_run);
                        sleep(Duration::from_millis(300));
                    }
                    Err(error) => return Err(error),
                }
            }

            if let Some(error) = last_error {
                return Err(error);
            }
            continue;
        }

        if is_dynamic_loop_attach {
            let outcome = run(&command, &args, plan.dry_run)?;
            if plan.dry_run {
                outcomes.push(outcome);
                continue;
            }

            let loop_device = outcome.stdout.trim().to_string();
            outcomes.push(outcome);
            if !loop_device.is_empty() {
                let partprobe_args = vec![loop_device.clone()];
                let _ = run("partprobe", &partprobe_args, false);
                let partx_args = vec!["-u".to_string(), loop_device];
                let _ = run("partx", &partx_args, false);
            }
            continue;
        }

        match run(&command, &args, plan.dry_run) {
            Ok(outcome) => outcomes.push(outcome),
            Err(TestingError::CommandFailed { stderr, .. })
                if command == "losetup"
                    && args.len() >= 2
                    && args[0] == "-d"
                    && stderr.contains("Permission denied") =>
            {
                outcomes.push(CommandOutcome {
                    command: step.clone(),
                    stdout: String::new(),
                    stderr,
                    executed: false,
                });
            }
            Err(error) => return Err(error),
        }
    }
    Ok(outcomes)
}

fn is_mountpoint(path: &str) -> bool {
    Command::new("findmnt")
        .args(["-rn", path])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn is_loop_attached(device: &str) -> bool {
    Command::new("losetup")
        .arg(device)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn ensure_partition_device_available(device: &str, dry_run: bool) -> Result<()> {
    if dry_run || Path::new(device).exists() {
        return Ok(());
    }

    let Some((loop_device, _)) = device.rsplit_once('p') else {
        return Ok(());
    };
    if !loop_device.starts_with("/dev/loop") {
        return Ok(());
    }

    for _ in 0..10 {
        let _ = Command::new("partprobe").arg(loop_device).status();
        let _ = Command::new("partx").args(["-u", loop_device]).status();

        let output = Command::new("lsblk")
            .args(["-lnpo", "NAME,MAJ:MIN", loop_device])
            .output()
            .map_err(|error| TestingError::CommandFailed {
                command: format!("lsblk -lnpo NAME,MAJ:MIN {}", loop_device),
                stderr: error.to_string(),
            })?;

        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let mut parts = line.split_whitespace();
                let Some(name) = parts.next() else {
                    continue;
                };
                if name != device {
                    continue;
                }
                if Path::new(device).exists() {
                    return Ok(());
                }

                if let Some((major, minor)) = parts.next().and_then(parse_major_minor) {
                    let status = Command::new("mknod")
                        .args([
                            "-m",
                            "660",
                            device,
                            "b",
                            &major.to_string(),
                            &minor.to_string(),
                        ])
                        .status()
                        .map_err(|error| TestingError::CommandFailed {
                            command: format!("mknod -m 660 {} b {} {}", device, major, minor),
                            stderr: error.to_string(),
                        })?;

                    if !status.success() {
                        return Err(TestingError::CommandFailed {
                            command: format!("mknod -m 660 {} b {} {}", device, major, minor),
                            stderr: format!("process exited with status {status}"),
                        });
                    }
                    return Ok(());
                }
            }
        }

        sleep(Duration::from_millis(200));
    }

    Err(TestingError::CommandFailed {
        command: format!("prepare block device {}", device),
        stderr: "partition device unavailable after retries".to_string(),
    })
}

fn ensure_loop_device_available(loop_device: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        return Ok(());
    }

    let in_use = Command::new("losetup")
        .arg(loop_device)
        .output()
        .map_err(|error| TestingError::CommandFailed {
            command: format!("losetup {}", loop_device),
            stderr: error.to_string(),
        })?
        .status
        .success();

    if !in_use {
        return Ok(());
    }

    let partition_output = Command::new("lsblk")
        .args(["-lnpo", "NAME", loop_device])
        .output()
        .map_err(|error| TestingError::CommandFailed {
            command: format!("lsblk -lnpo NAME {}", loop_device),
            stderr: error.to_string(),
        })?;

    if partition_output.status.success() {
        for partition in String::from_utf8_lossy(&partition_output.stdout)
            .lines()
            .skip(1)
        {
            let mount_output = Command::new("findmnt")
                .args(["-rn", "-S", partition, "-o", "TARGET"])
                .output()
                .map_err(|error| TestingError::CommandFailed {
                    command: format!("findmnt -rn -S {} -o TARGET", partition),
                    stderr: error.to_string(),
                })?;

            if mount_output.status.success() {
                for target in String::from_utf8_lossy(&mount_output.stdout).lines() {
                    let status = Command::new("umount")
                        .arg(target)
                        .status()
                        .map_err(|error| TestingError::CommandFailed {
                            command: format!("umount {}", target),
                            stderr: error.to_string(),
                        })?;
                    if !status.success() {
                        return Err(TestingError::CommandFailed {
                            command: format!("umount {}", target),
                            stderr: format!("process exited with status {status}"),
                        });
                    }
                }
            }
        }
    }

    let detach_status = Command::new("losetup")
        .args(["-d", loop_device])
        .status()
        .map_err(|error| TestingError::CommandFailed {
            command: format!("losetup -d {}", loop_device),
            stderr: error.to_string(),
        })?;

    if !detach_status.success() {
        return Err(TestingError::CommandFailed {
            command: format!("losetup -d {}", loop_device),
            stderr: format!("process exited with status {detach_status}"),
        });
    }

    Ok(())
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
    let mut mapped_partitions = Vec::new();
    for loop_device in loops {
        let discovered = discover_partitions(loop_device)?;
        mapped_partitions.extend(discovered);
    }
    state.mapped_partitions = mapped_partitions;

    ledger::save(&state)?;
    Ok(())
}

fn discover_partitions(loop_device: &str) -> Result<Vec<String>> {
    for _ in 0..10 {
        let _ = Command::new("partprobe").arg(loop_device).status();

        let output = Command::new("lsblk")
            .args(["-lnpo", "NAME,MAJ:MIN", loop_device])
            .output()
            .map_err(|error| TestingError::CommandFailed {
                command: format!("lsblk -lnpo NAME,MAJ:MIN {}", loop_device),
                stderr: error.to_string(),
            })?;

        if !output.status.success() {
            sleep(Duration::from_millis(200));
            continue;
        }

        let mut partitions = Vec::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let mut parts = line.split_whitespace();
            let Some(name) = parts.next() else {
                continue;
            };
            let major_minor = parts.next();
            if name == loop_device {
                continue;
            }

            if !Path::new(name).exists()
                && let Some((major, minor)) = major_minor.and_then(parse_major_minor)
            {
                let status = Command::new("mknod")
                    .args([
                        "-m",
                        "660",
                        name,
                        "b",
                        &major.to_string(),
                        &minor.to_string(),
                    ])
                    .status()
                    .map_err(|error| TestingError::CommandFailed {
                        command: format!("mknod -m 660 {} b {} {}", name, major, minor),
                        stderr: error.to_string(),
                    })?;

                if !status.success() {
                    return Err(TestingError::CommandFailed {
                        command: format!("mknod -m 660 {} b {} {}", name, major, minor),
                        stderr: format!("process exited with status {status}"),
                    });
                }
            }

            partitions.push(name.to_string());
        }

        if !partitions.is_empty() {
            return Ok(partitions);
        }

        sleep(Duration::from_millis(200));
    }

    Err(TestingError::SpecInvalid {
        spec_name: loop_device.to_string(),
        reason: "partition device discovery timed out".to_string(),
    })
}

fn parse_major_minor(value: &str) -> Option<(u32, u32)> {
    let (major, minor) = value.split_once(':')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
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
