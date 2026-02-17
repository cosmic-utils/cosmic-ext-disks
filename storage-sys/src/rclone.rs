// SPDX-License-Identifier: GPL-3.0-only

//! Low-level RClone CLI operations
//!
//! This module provides wrappers around the rclone command-line tool
//! for listing remotes, reading configuration, and managing mounts.

use crate::error::{Result, SysError};
use configparser::ini::Ini;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use storage_common::ConfigScope;
use tracing::{debug, info, warn};
use which::which;

/// RClone CLI wrapper for low-level operations
pub struct RCloneCli {
    /// Path to the rclone binary
    binary_path: PathBuf,
}

impl RCloneCli {
    /// Create a new RClone CLI wrapper
    ///
    /// Returns an error if rclone is not installed
    pub fn new() -> Result<Self> {
        let binary_path = Self::find_rclone_binary()?;
        info!("Found rclone binary at {:?}", binary_path);
        Ok(Self { binary_path })
    }

    /// Find the rclone binary in PATH
    pub fn find_rclone_binary() -> Result<PathBuf> {
        which("rclone").map_err(|_| SysError::RCloneNotFound)
    }

    /// Get the config file path for a given scope
    pub fn get_config_path(scope: ConfigScope) -> PathBuf {
        scope.config_path()
    }

    /// List all configured remotes using `rclone listremotes`
    pub fn list_remotes(&self, config_path: &PathBuf) -> Result<Vec<String>> {
        debug!("Listing remotes from {:?}", config_path);

        let output = Command::new(&self.binary_path)
            .arg("listremotes")
            .arg("--config")
            .arg(config_path)
            .output()
            .map_err(|e| SysError::OperationFailed(format!("Failed to execute rclone: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("rclone listremotes failed: {}", stderr);
            return Err(SysError::RCloneConfigParse(format!(
                "Failed to list remotes: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let remotes: Vec<String> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.trim().trim_end_matches(':').to_string())
            .collect();

        debug!("Found {} remotes", remotes.len());
        Ok(remotes)
    }

    /// Read and parse the rclone configuration file
    pub fn read_config(
        &self,
        config_path: &PathBuf,
    ) -> Result<HashMap<String, HashMap<String, Option<String>>>> {
        debug!("Reading config from {:?}", config_path);

        if !config_path.exists() {
            return Err(SysError::RCloneConfigNotFound);
        }

        // Read the file content first to provide better error messages
        let content = std::fs::read_to_string(config_path).map_err(|e| {
            SysError::RCloneConfigParse(format!("Failed to read configuration file: {}", e))
        })?;

        // Check if the file is empty
        if content.trim().is_empty() {
            warn!("Configuration file is empty: {:?}", config_path);
            return Ok(HashMap::new());
        }

        let mut conf = Ini::new();
        match conf.read(content) {
            Ok(_) => {
                let remotes = conf.get_map_ref().clone();
                debug!("Parsed {} remote sections", remotes.keys().count());
                Ok(remotes)
            }
            Err(e) => {
                warn!("Failed to parse rclone config: {}", e);
                Err(SysError::RCloneConfigParse(format!(
                    "Failed to parse configuration: {}",
                    e
                )))
            }
        }
    }

    /// Get the mount point for a remote with a given scope
    pub fn get_mount_point(remote_name: &str, scope: ConfigScope) -> PathBuf {
        scope.mount_point(remote_name)
    }

    /// Check if a mount point is currently mounted
    pub fn is_mounted(mount_point: &PathBuf) -> Result<bool> {
        debug!("Checking if {:?} is mounted", mount_point);

        if !mount_point.exists() {
            return Ok(false);
        }

        let output = Command::new("mountpoint")
            .arg("-q")
            .arg(mount_point)
            .output()
            .map_err(|e| SysError::OperationFailed(format!("Failed to run mountpoint: {}", e)))?;

        Ok(output.status.success())
    }

    /// Mount a remote using `rclone mount`
    ///
    /// This runs rclone in daemon mode (--daemon) which forks into the background
    pub fn mount(
        &self,
        remote_name: &str,
        mount_point: &PathBuf,
        config_path: &PathBuf,
        scope: ConfigScope,
        uid: Option<u32>,
    ) -> Result<()> {
        info!("Mounting remote {} at {:?}", remote_name, mount_point);

        // Check if already mounted
        if Self::is_mounted(mount_point)? {
            return Err(SysError::RCloneAlreadyMounted(remote_name.to_string()));
        }

        // Create mount point if it doesn't exist
        if !mount_point.exists() {
            std::fs::create_dir_all(mount_point).map_err(|e| {
                SysError::OperationFailed(format!("Failed to create mount point: {}", e))
            })?;
        }

        // Ensure user-owned mountpoint for user scope
        if scope == ConfigScope::User {
            if let Some(uid) = uid {
                if unsafe { libc::geteuid() } == 0 {
                    if let Some((uid, gid)) = uid_gid_for_uid(uid) {
                        if let Some(parent) = mount_point.parent() {
                            chown_path(parent, uid, gid)?;
                        }
                        chown_path(mount_point, uid, gid)?;
                    } else {
                        warn!("Failed to resolve uid/gid for uid {}", uid);
                    }
                } else {
                    debug!("Skipping mountpoint chown; not running as root");
                }
            }
        }

        let remote_path = format!("{}:", remote_name);

        let output = Command::new(&self.binary_path)
            .arg("mount")
            .arg(&remote_path)
            .arg(mount_point)
            .arg("--config")
            .arg(config_path)
            .arg("--daemon")
            .arg("--vfs-cache-mode")
            .arg("writes")
            .output()
            .map_err(|e| {
                SysError::RCloneMountFailed(format!("Failed to execute rclone mount: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("rclone mount failed: {}", stderr);
            return Err(SysError::RCloneMountFailed(format!(
                "Mount failed for {}: {}",
                remote_name, stderr
            )));
        }

        info!("Successfully mounted {} at {:?}", remote_name, mount_point);
        Ok(())
    }

    /// Unmount a remote using fusermount
    pub fn unmount(&self, mount_point: &PathBuf) -> Result<()> {
        info!("Unmounting {:?}", mount_point);

        // Check if mounted
        if !Self::is_mounted(mount_point)? {
            return Err(SysError::RCloneNotMounted(
                mount_point.display().to_string(),
            ));
        }

        let output = Command::new("fusermount")
            .arg("-u")
            .arg(mount_point)
            .output()
            .map_err(|e| {
                SysError::RCloneUnmountFailed(format!("Failed to execute fusermount: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("fusermount failed: {}", stderr);
            return Err(SysError::RCloneUnmountFailed(format!(
                "Unmount failed for {:?}: {}",
                mount_point, stderr
            )));
        }

        info!("Successfully unmounted {:?}", mount_point);
        Ok(())
    }

    /// Test a remote configuration using `rclone ls`
    ///
    /// Returns (success, message, latency_ms)
    pub fn test_remote(
        &self,
        remote_name: &str,
        config_path: &PathBuf,
    ) -> Result<(bool, String, u64)> {
        info!(
            "Testing remote {} with config {:?}",
            remote_name, config_path
        );

        let remote_path = format!("{}:", remote_name);
        let start = Instant::now();

        let output = Command::new(&self.binary_path)
            .arg("ls")
            .arg(&remote_path)
            .arg("--config")
            .arg(config_path)
            .arg("--max-depth")
            .arg("1")
            .output()
            .map_err(|e| {
                SysError::RCloneTestFailed(format!("Failed to execute rclone ls: {}", e))
            })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if output.status.success() {
            info!("Remote {} test succeeded in {}ms", remote_name, latency_ms);
            Ok((true, "Connection successful".to_string(), latency_ms))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let message = if stderr.is_empty() {
                "Connection failed".to_string()
            } else {
                stderr.to_string()
            };
            warn!("Remote {} test failed: {}", remote_name, message);
            Ok((false, message, latency_ms))
        }
    }

    /// Write configuration back to file
    pub fn write_config(
        &self,
        config_path: &PathBuf,
        remotes: &HashMap<String, HashMap<String, Option<String>>>,
    ) -> Result<()> {
        info!("Writing config to {:?}", config_path);

        let mut conf = Ini::new();

        for (section, properties) in remotes.iter() {
            for (key, value) in properties.iter() {
                conf.set(section, key, value.clone());
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    SysError::OperationFailed(format!("Failed to create config directory: {}", e))
                })?;
            }
        }

        conf.write(config_path)
            .map_err(|e| SysError::RCloneConfigParse(format!("Failed to write config: {}", e)))?;

        info!("Successfully wrote config to {:?}", config_path);
        Ok(())
    }
}

impl std::fmt::Debug for RCloneCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RCloneCli")
            .field("binary_path", &self.binary_path)
            .finish()
    }
}

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

fn username_for_uid(uid: u32) -> Option<String> {
    unsafe {
        let pw = libc::getpwuid(uid);
        if pw.is_null() {
            return None;
        }
        let name = std::ffi::CStr::from_ptr((*pw).pw_name);
        name.to_str().ok().map(|name| name.to_string())
    }
}

fn uid_gid_for_uid(uid: u32) -> Option<(u32, u32)> {
    unsafe {
        let pw = libc::getpwuid(uid);
        if pw.is_null() {
            return None;
        }
        Some(((*pw).pw_uid as u32, (*pw).pw_gid as u32))
    }
}

fn chown_path(path: &Path, uid: u32, gid: u32) -> Result<()> {
    let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|e| {
        SysError::OperationFailed(format!("Invalid path for chown {}: {}", path.display(), e))
    })?;
    let result = unsafe { libc::chown(c_path.as_ptr(), uid as libc::uid_t, gid as libc::gid_t) };
    if result != 0 {
        return Err(SysError::OperationFailed(format!(
            "Failed to chown {}: {}",
            path.display(),
            std::io::Error::last_os_error()
        )));
    }
    Ok(())
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

pub fn set_mount_on_boot(
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

pub fn is_mount_on_boot_enabled(
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
