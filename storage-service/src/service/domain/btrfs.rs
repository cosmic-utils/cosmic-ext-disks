// SPDX-License-Identifier: GPL-3.0-only

pub trait BtrfsDomain: Send + Sync {
    fn require_available(&self) -> zbus::fdo::Result<()>;
}

pub struct DefaultBtrfsDomain;

impl BtrfsDomain for DefaultBtrfsDomain {
    fn require_available(&self) -> zbus::fdo::Result<()> {
        if !cfg!(feature = "btrfs-tools") {
            return Err(zbus::fdo::Error::Failed(
                "Btrfs unavailable: compile-time feature disabled".to_string(),
            ));
        }

        Ok(())
    }
}
