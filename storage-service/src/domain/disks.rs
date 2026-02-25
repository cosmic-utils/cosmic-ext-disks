// SPDX-License-Identifier: GPL-3.0-only

use storage_types::DiskInfo;

pub trait DisksDomain: Send + Sync {
    fn disk_matches(&self, disk: &DiskInfo, requested: &str) -> bool;
}

pub struct DisksPolicy;

impl DisksDomain for DisksPolicy {
    fn disk_matches(&self, disk: &DiskInfo, requested: &str) -> bool {
        let device_name = requested.strip_prefix("/dev/").unwrap_or(requested);

        if disk.device == requested {
            return true;
        }

        if let Some(disk_name) = disk.device.rsplit('/').next()
            && disk_name == device_name
        {
            return true;
        }

        disk.id == requested || disk.id == device_name
    }
}
