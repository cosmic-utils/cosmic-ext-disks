// SPDX-License-Identifier: GPL-3.0-only

pub trait PartitionsDomain: Send + Sync {
    fn normalize_table_type(&self, table_type: &str) -> zbus::fdo::Result<String>;
    fn normalize_disk_device(&self, disk: &str) -> String;
}

pub struct PartitionsPolicy;

impl PartitionsDomain for PartitionsPolicy {
    fn normalize_table_type(&self, table_type: &str) -> zbus::fdo::Result<String> {
        match table_type.to_lowercase().as_str() {
            "gpt" => Ok("gpt".to_string()),
            "dos" | "mbr" | "msdos" => Ok("dos".to_string()),
            _ => Err(zbus::fdo::Error::InvalidArgs(format!(
                "Invalid table type: {table_type}. Must be 'gpt' or 'dos'/'mbr'"
            ))),
        }
    }

    fn normalize_disk_device(&self, disk: &str) -> String {
        if disk.starts_with("/dev/") {
            disk.to_string()
        } else {
            format!("/dev/{disk}")
        }
    }
}
