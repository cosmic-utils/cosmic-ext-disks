use std::process::Command;

use crate::errors::{Result, TestingError};

#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub executed: bool,
}

pub fn render(command: &str, args: &[String]) -> String {
    if args.is_empty() {
        command.to_string()
    } else {
        format!("{} {}", command, args.join(" "))
    }
}

pub fn run(command: &str, args: &[String], dry_run: bool) -> Result<CommandOutcome> {
    let rendered = render(command, args);
    if dry_run {
        return Ok(CommandOutcome {
            command: rendered,
            stdout: String::new(),
            stderr: String::new(),
            executed: false,
        });
    }

    let output =
        Command::new(command)
            .args(args)
            .output()
            .map_err(|error| TestingError::CommandFailed {
                command: rendered.clone(),
                stderr: error.to_string(),
            })?;

    if !output.status.success() {
        return Err(TestingError::CommandFailed {
            command: rendered,
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    Ok(CommandOutcome {
        command: rendered,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        executed: true,
    })
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn formats_command_context() {
        let args = vec![
            "--find".to_string(),
            "--show".to_string(),
            "disk.img".to_string(),
        ];
        let rendered = render("losetup", &args);
        assert!(rendered.contains("losetup --find --show disk.img"));
    }
}
