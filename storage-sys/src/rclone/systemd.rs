use crate::error::{Result, SysError};
use crate::rclone::RCloneCli;
use crate::rclone::unix_user::username_for_uid;
use std::path::PathBuf;
use std::process::Command;
use storage_types::ConfigScope;
use which::which;

const SYSTEMD_UNIT_PREFIX: &str = "storage-rclone-mount";

fn systemd_unit_name(remote_name: &str) -> String {
    format!("{}@{}.service", SYSTEMD_UNIT_PREFIX, remote_name)
}

fn systemd_template_name() -> String {
    format!("{}@.service", SYSTEMD_UNIT_PREFIX)
}

fn systemd_unit_dir(scope: ConfigScope, home: Option<&std::path::Path>) -> Result<PathBuf> {
    match scope {
        ConfigScope::User => {
            let home = home.ok_or_else(|| {
                SysError::OperationFailed("Missing home directory for user scope".to_string())
            })?;
            Ok(home.join(".config/systemd/user"))
        }
        ConfigScope::System => Ok(PathBuf::from("/etc/systemd/system")),
    }
}

fn systemctl_command(
    scope: ConfigScope,
    uid: Option<u32>,
    home: Option<&std::path::Path>,
) -> Command {
    let mut command = Command::new("systemctl");
    if scope == ConfigScope::User {
        command.arg("--user");
        if let Some(uid) = uid {
            if let Some(username) = username_for_uid(uid) {
                command.arg("--machine").arg(format!("{username}@.host"));
            }
            command.env("XDG_RUNTIME_DIR", format!("/run/user/{uid}"));
        }
        if let Some(home) = home {
            command.env("HOME", home);
        }
    }
    command
}

fn run_systemctl(
    scope: ConfigScope,
    uid: Option<u32>,
    home: Option<&std::path::Path>,
    args: &[&str],
) -> Result<String> {
    let output = systemctl_command(scope, uid, home)
        .args(args)
        .output()
        .map_err(|e| SysError::OperationFailed(format!("Failed to run systemctl: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SysError::OperationFailed(format!(
            "systemctl failed: {}",
            stderr
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn build_unit_contents(
    scope: ConfigScope,
    rclone_path: &std::path::Path,
    mkdir_path: &std::path::Path,
    fusermount_path: &std::path::Path,
) -> String {
    let (mount_prefix, config_path, wanted_by) = match scope {
        ConfigScope::User => (
            "%h/mnt".to_string(),
            "%h/.config/rclone/rclone.conf".to_string(),
            "default.target".to_string(),
        ),
        ConfigScope::System => (
            "/mnt/rclone".to_string(),
            "/etc/rclone.conf".to_string(),
            "multi-user.target".to_string(),
        ),
    };

    format!(
        "[Unit]\n\
Description=RClone mount for %i\n\
After=network-online.target\n\
Wants=network-online.target\n\n\
[Service]\n\
Type=simple\n\
ExecStartPre={} -p {}/%i\n\
ExecStart={} mount %i: {}/%i --config {} --vfs-cache-mode writes\n\
ExecStop={} -u {}/%i\n\
Restart=on-failure\n\
RestartSec=5\n\n\
[Install]\n\
WantedBy={}\n",
        mkdir_path.display(),
        mount_prefix,
        rclone_path.display(),
        mount_prefix,
        config_path,
        fusermount_path.display(),
        mount_prefix,
        wanted_by
    )
}

fn ensure_systemd_template(
    scope: ConfigScope,
    home: Option<&std::path::Path>,
    uid: Option<u32>,
) -> Result<PathBuf> {
    let unit_dir = systemd_unit_dir(scope, home)?;
    if !unit_dir.exists() {
        std::fs::create_dir_all(&unit_dir).map_err(SysError::Io)?;
    }

    let unit_path = unit_dir.join(systemd_template_name());

    let rclone_path = RCloneCli::find_rclone_binary()?;
    let mkdir_path = which("mkdir")
        .map_err(|e| SysError::OperationFailed(format!("Failed to locate mkdir: {}", e)))?;
    let fusermount_path = which("fusermount3")
        .or_else(|_| which("fusermount"))
        .map_err(|e| SysError::OperationFailed(format!("Failed to locate fusermount: {}", e)))?;

    let contents = build_unit_contents(scope, &rclone_path, &mkdir_path, &fusermount_path);

    let needs_write = match std::fs::read_to_string(&unit_path) {
        Ok(existing) => existing != contents,
        Err(_) => true,
    };

    if needs_write {
        std::fs::write(&unit_path, contents).map_err(SysError::Io)?;
        run_systemctl(scope, uid, home, &["daemon-reload"])?;
    }

    Ok(unit_path)
}

pub(crate) fn set_mount_on_boot(
    scope: ConfigScope,
    remote_name: &str,
    enabled: bool,
    uid: Option<u32>,
    home: Option<&std::path::Path>,
) -> Result<()> {
    ensure_systemd_template(scope, home, uid)?;

    let unit_name = systemd_unit_name(remote_name);
    if enabled {
        run_systemctl(scope, uid, home, &["enable", "--now", &unit_name])?;
    } else {
        run_systemctl(scope, uid, home, &["disable", "--now", &unit_name])?;
    }

    Ok(())
}

pub(crate) fn is_mount_on_boot_enabled(
    scope: ConfigScope,
    remote_name: &str,
    uid: Option<u32>,
    home: Option<&std::path::Path>,
) -> Result<bool> {
    let unit_name = systemd_unit_name(remote_name);
    let output = systemctl_command(scope, uid, home)
        .args(["is-enabled", &unit_name])
        .output()
        .map_err(|e| SysError::OperationFailed(format!("Failed to run systemctl: {}", e)))?;

    if !output.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let status = stdout.trim();
    Ok(status == "enabled" || status == "enabled-runtime")
}
