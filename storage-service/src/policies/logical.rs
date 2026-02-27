// SPDX-License-Identifier: GPL-3.0-only

pub trait LogicalDomain: Send + Sync {
    fn require_read(&self) -> zbus::fdo::Result<()>;
    fn require_modify(&self) -> zbus::fdo::Result<()>;
}

pub struct LogicalPolicy;

impl LogicalDomain for LogicalPolicy {
    fn require_read(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }

    fn require_modify(&self) -> zbus::fdo::Result<()> {
        Ok(())
    }
}
