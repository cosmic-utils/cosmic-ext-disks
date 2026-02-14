mod dbus;
mod disks;
mod operations;
mod options;
mod udisks_block_config;
mod usage;

// New domain-based modules (GAP-001.b)
pub mod disk;
pub mod partition;
pub mod filesystem;
pub mod encryption;
pub mod image;
pub mod smart;
pub mod lvm;
pub mod btrfs;
pub mod gpt;
pub mod manager;
pub mod volume;
pub mod util;

// Re-export storage-models types (canonical domain models)
pub use storage_models;
pub use storage_models::ProcessInfo;

// Re-export format utilities (now in storage-models)
pub use storage_models::{bytes_to_pretty, get_numeric, get_step, pretty_to_bytes};

// Re-export partition type catalog (now in storage-models)
pub use storage_models::{
    COMMON_DOS_TYPES, COMMON_GPT_TYPES, PartitionTypeInfo, PartitionTypeInfoFlags,
    get_all_partition_type_infos, get_valid_partition_names,
};

// Re-export volume types (now in storage-models)
pub use storage_models::{VolumeKind, VolumeType};

// Re-export CreatePartitionInfo (now in storage-models)
pub use storage_models::CreatePartitionInfo;

// Re-export GPT alignment (now in storage-models)
pub use storage_models::GPT_ALIGNMENT_BYTES;

// Re-export commonly used zbus types
pub use zbus::zvariant::OwnedObjectPath;

// Explicit exports from dbus module (DBus byte string encoding/decoding)
pub use dbus::bytestring::{
    bytestring_owned_value, decode_c_string_bytes, decode_mount_points, encode_bytestring,
    owned_value_to_bytestring,
};

// Re-export key types from new modules
pub use disk::model::DriveModel;
pub use manager::{DiskManager, DeviceEvent, DeviceEventStream};
pub use smart::{SmartInfo, SmartSelfTestKind};
pub use btrfs::{BtrfsFilesystem, BtrfsSubvolume};

// Re-export disks module types for backwards compatibility
pub use disks::{
    DiskError, EncryptionOptionsSettings, MountOptionsSettings,
    VolumeModel, VolumeNode,
};

// Re-export operations from new domain modules
pub use gpt::{fallback_gpt_usable_range_bytes, probe_gpt_usable_range_bytes};
pub use lvm::list_lvs_for_pv;
pub use util::{find_processes_using_mount, kill_processes};
pub use image::{loop_setup, open_for_backup, open_for_restore};

// Partition operations (from new partition module)
pub use partition::{
    create_partition_table, create_partition, delete_partition,
    resize_partition, set_partition_type, set_partition_flags,
    set_partition_name, edit_partition,
};

// Filesystem operations (from new filesystem module)
pub use filesystem::{
    format_filesystem, mount_filesystem, unmount_filesystem,
    check_filesystem, repair_filesystem, get_filesystem_label,
    set_filesystem_label, take_filesystem_ownership, get_mount_point,
};

// Encryption operations (from new encryption module)
pub use encryption::{
    unlock_luks, lock_luks, change_luks_passphrase,
    format_luks, list_luks_devices,
};

// SMART operations (from new smart module)
pub use smart::{get_drive_smart_info, start_drive_smart_selftest, abort_drive_smart_selftest};

// Disk operations (from new disk module)
pub use disk::{
    eject_drive, power_off_drive, standby_drive, wakeup_drive,
    remove_drive, format_disk,
};

// Explicit exports from options module (mount/encryption option parsing)
pub use options::{
    join_options, merge_other_with_managed, normalize_options, remove_prefixed, remove_token,
    set_prefixed_value, set_token_present, split_options, stable_dedup,
};

// Explicit exports from udisks_block_config module (UDisks2 configuration helpers)
pub use udisks_block_config::{ConfigurationItem, UDisks2BlockConfigurationProxy};

// Explicit exports from usage module (filesystem usage statistics)
pub use usage::{Usage, usage_for_mount_point};

// Legacy: Re-export old operations module for backwards compatibility
// (These now delegate to new domain modules)
#[deprecated(note = "Use operations from domain modules (partition::*, filesystem::*, etc.) instead")]
pub mod legacy_operations {
    pub use crate::operations::*;
}
