// SPDX-License-Identifier: GPL-3.0-only

use storage_common::rclone::{ConfigScope, RemoteConfig, rclone_provider};

pub trait RcloneDomain: Send + Sync {
    fn require_available(&self) -> zbus::fdo::Result<()>;
    fn parse_scope(&self, scope: &str) -> zbus::fdo::Result<ConfigScope>;
    fn validate_remote_config(&self, config: &RemoteConfig) -> zbus::fdo::Result<()>;
}

pub struct DefaultRcloneDomain;

impl RcloneDomain for DefaultRcloneDomain {
    fn require_available(&self) -> zbus::fdo::Result<()> {
        if !cfg!(feature = "rclone-tools") {
            return Err(zbus::fdo::Error::Failed(
                "RClone unavailable: compile-time feature disabled".to_string(),
            ));
        }

        Ok(())
    }

    fn parse_scope(&self, scope: &str) -> zbus::fdo::Result<ConfigScope> {
        self.require_available()?;
        scope
            .parse()
            .map_err(|e| zbus::fdo::Error::Failed(format!("Invalid scope: {}", e)))
    }

    fn validate_remote_config(&self, config: &RemoteConfig) -> zbus::fdo::Result<()> {
        self.require_available()?;
        config.validate_name().map_err(zbus::fdo::Error::Failed)?;

        let provider = rclone_provider(&config.remote_type).ok_or_else(|| {
            zbus::fdo::Error::Failed(format!("Unsupported remote type: {}", config.remote_type))
        })?;

        for option in &provider.options {
            if !option.required || option.is_hidden() {
                continue;
            }
            let value = config.options.get(&option.name).map(|v| v.trim());
            if value.is_none() || value == Some("") {
                return Err(zbus::fdo::Error::Failed(format!(
                    "Missing required field '{}' for {}",
                    option.name, config.remote_type
                )));
            }
        }

        Ok(())
    }
}
