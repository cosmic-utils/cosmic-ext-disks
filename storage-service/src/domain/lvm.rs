// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

pub trait LvmDomain: Send + Sync {
    fn require_lvm(&self) -> zbus::fdo::Result<()>;
}

pub struct DefaultLvmDomain {
    lvm_available: bool,
}

impl DefaultLvmDomain {
    pub fn new() -> Self {
        let lvm_available = cfg!(feature = "lvm-tools")
            && Path::new("/sbin/pvs").exists()
            && Path::new("/sbin/vgs").exists()
            && Path::new("/sbin/lvs").exists();

        Self { lvm_available }
    }
}

impl LvmDomain for DefaultLvmDomain {
    fn require_lvm(&self) -> zbus::fdo::Result<()> {
        if !cfg!(feature = "lvm-tools") {
            return Err(zbus::fdo::Error::Failed(
                "LVM unavailable: compile-time feature disabled".to_string(),
            ));
        }

        if !self.lvm_available {
            return Err(zbus::fdo::Error::Failed(
                "LVM tools not available on this system".to_string(),
            ));
        }

        Ok(())
    }
}
