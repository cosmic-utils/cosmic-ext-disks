use which::which;

use crate::errors::{Result, TestingError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    Auto,
    Podman,
    Docker,
}

impl Runtime {
    pub fn name(self) -> &'static str {
        match self {
            Runtime::Auto => "auto",
            Runtime::Podman => "podman",
            Runtime::Docker => "docker",
        }
    }

    pub fn binary(self) -> &'static str {
        match self {
            Runtime::Auto => "",
            Runtime::Podman => "podman",
            Runtime::Docker => "docker",
        }
    }
}

pub fn resolve(selection: Option<&str>) -> Result<Runtime> {
    match selection.unwrap_or("auto") {
        "podman" => {
            if which("podman").is_ok() {
                Ok(Runtime::Podman)
            } else {
                Err(TestingError::RuntimeMissing)
            }
        }
        "docker" => {
            if which("docker").is_ok() {
                Ok(Runtime::Docker)
            } else {
                Err(TestingError::RuntimeMissing)
            }
        }
        "auto" => {
            if which("podman").is_ok() {
                Ok(Runtime::Podman)
            } else if which("docker").is_ok() {
                Ok(Runtime::Docker)
            } else {
                Err(TestingError::RuntimeMissing)
            }
        }
        _ => Err(TestingError::RuntimeMissing),
    }
}

pub fn build_image_args(tag: &str, containerfile: &str, context_dir: &str) -> Vec<String> {
    vec![
        "build".to_string(),
        "-t".to_string(),
        tag.to_string(),
        "-f".to_string(),
        containerfile.to_string(),
        context_dir.to_string(),
    ]
}

pub fn run_privileged_args(name: &str, image: &str, command: &[String]) -> Vec<String> {
    let mut args = vec![
        "run".to_string(),
        "--name".to_string(),
        name.to_string(),
        "--privileged".to_string(),
        "--rm".to_string(),
        image.to_string(),
    ];
    args.extend_from_slice(command);
    args
}

pub fn exec_args(name: &str, command: &[String]) -> Vec<String> {
    let mut args = vec!["exec".to_string(), name.to_string()];
    args.extend_from_slice(command);
    args
}

pub fn rm_args(name: &str, force: bool) -> Vec<String> {
    let mut args = vec!["rm".to_string()];
    if force {
        args.push("-f".to_string());
    }
    args.push(name.to_string());
    args
}

#[cfg(test)]
mod tests {
    use super::{Runtime, rm_args};

    #[test]
    fn explicit_runtime_selection_is_respected() {
        let runtime = Runtime::Docker;
        assert_eq!(runtime.name(), "docker");
    }

    #[test]
    fn rm_args_include_force_when_requested() {
        let args = rm_args("storage-testing", true);
        assert_eq!(args, vec!["rm", "-f", "storage-testing"]);
    }
}
