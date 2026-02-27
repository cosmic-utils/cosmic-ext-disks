use crate::artifacts;
use crate::cmd::{CommandOutcome, run};
use crate::errors::{Result, TestingError};
use crate::runtime::{build_image_args, resolve, rm_args};

#[derive(Debug, Clone)]
struct PlannedCommand {
    command: String,
    args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub steps: Vec<String>,
    pub dry_run: bool,
    commands: Vec<PlannedCommand>,
}

pub fn plan_run(suite: &str, runtime: &str, keep: bool, dry_run: bool) -> Result<Plan> {
    let runtime = resolve(Some(runtime))?;
    let runtime_bin = runtime.binary().to_string();
    let workspace =
        std::env::current_dir().map_err(|error| TestingError::ContainerRuntimeFailed {
            reason: error.to_string(),
        })?;

    let mut steps = Vec::new();
    let mut commands = Vec::new();
    let containerfile = "storage-testing/container/Containerfile";
    let image_tag = "storage-testing:local";
    let container_name = "storage-testing-harness";

    let build_args = build_image_args(image_tag, containerfile, ".");
    steps.push(format!("{} {}", runtime_bin, build_args.join(" ")));
    commands.push(PlannedCommand {
        command: runtime_bin.clone(),
        args: build_args,
    });

    let run_script = format!(
        "set -euo pipefail; mkdir -p /run/dbus; dbus-daemon --system --fork --nopidfile; if command -v polkitd >/dev/null 2>&1; then polkitd --no-debug & elif [[ -x /usr/lib/polkit-1/polkitd ]]; then /usr/lib/polkit-1/polkitd --no-debug & fi; ./target/debug/cosmic-ext-storage-service > /tmp/storage-service.log 2>&1 & for i in $(seq 1 20); do if dbus-send --system --dest=org.freedesktop.DBus --type=method_call --print-reply /org/freedesktop/DBus org.freedesktop.DBus.NameHasOwner string:org.cosmic.ext.Storage.Service | grep -q 'boolean true'; then break; fi; sleep 1; done; dbus-send --system --dest=org.freedesktop.DBus --type=method_call --print-reply /org/freedesktop/DBus org.freedesktop.DBus.NameHasOwner string:org.cosmic.ext.Storage.Service | grep -q 'boolean true'; cargo test -p storage-testing --test {}",
        suite
    );

    let run_args = vec![
        "run".to_string(),
        "--name".to_string(),
        container_name.to_string(),
        "--privileged".to_string(),
        "--rm".to_string(),
        "-v".to_string(),
        format!("{}:/workspace", workspace.display()),
        "-w".to_string(),
        "/workspace".to_string(),
        image_tag.to_string(),
        "bash".to_string(),
        "-lc".to_string(),
        run_script,
    ];

    steps.push(format!("{} {}", runtime_bin, run_args.join(" ")));
    commands.push(PlannedCommand {
        command: runtime_bin.clone(),
        args: run_args,
    });

    if !keep {
        let remove_args = rm_args(container_name, true);
        steps.push(format!("{} {}", runtime_bin, remove_args.join(" ")));
        commands.push(PlannedCommand {
            command: runtime_bin,
            args: remove_args,
        });
    }

    Ok(Plan {
        dry_run,
        steps,
        commands,
    })
}

pub fn plan_shell(runtime: &str) -> Result<Plan> {
    let runtime = resolve(Some(runtime))?;
    let workspace =
        std::env::current_dir().map_err(|error| TestingError::ContainerRuntimeFailed {
            reason: error.to_string(),
        })?;
    let runtime_bin = runtime.binary().to_string();
    let shell_args = vec![
        "run".to_string(),
        "--privileged".to_string(),
        "-it".to_string(),
        "--rm".to_string(),
        "-v".to_string(),
        format!("{}:/workspace", workspace.display()),
        "-w".to_string(),
        "/workspace".to_string(),
        "storage-testing:local".to_string(),
        "sh".to_string(),
    ];
    Ok(Plan {
        dry_run: false,
        steps: vec![format!("{} {}", runtime_bin, shell_args.join(" "))],
        commands: vec![PlannedCommand {
            command: runtime_bin,
            args: shell_args,
        }],
    })
}

pub fn plan_cleanup(runtime: &str) -> Result<Plan> {
    let runtime = resolve(Some(runtime))?;
    let runtime_bin = runtime.binary().to_string();
    let remove_args = rm_args("storage-testing-harness", true);
    Ok(Plan {
        dry_run: false,
        steps: vec![format!("{} {}", runtime_bin, remove_args.join(" "))],
        commands: vec![PlannedCommand {
            command: runtime_bin,
            args: remove_args,
        }],
    })
}

pub fn init_artifacts() -> Result<std::path::PathBuf> {
    artifacts::run_dir("harness").map_err(|error| TestingError::ContainerRuntimeFailed {
        reason: error.to_string(),
    })
}

pub fn execute_plan(plan: &Plan) -> Result<Vec<CommandOutcome>> {
    let mut outcomes = Vec::new();
    for command in &plan.commands {
        outcomes.push(run(&command.command, &command.args, plan.dry_run)?);
    }
    Ok(outcomes)
}

#[cfg(test)]
mod tests {
    use super::plan_run;
    use which::which;

    #[test]
    fn harness_run_builds_expected_runtime_plan() {
        let runtime = if which("podman").is_ok() {
            "podman"
        } else if which("docker").is_ok() {
            "docker"
        } else {
            return;
        };
        let plan = plan_run("harness_smoke", runtime, false, false).unwrap();
        assert!(
            plan.steps
                .iter()
                .any(|step| step.contains("run --name") && step.contains("--privileged"))
        );
    }
}
