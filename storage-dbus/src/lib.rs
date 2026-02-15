mod dbus;
mod options;
mod udisks_block_config;
mod usage;

// Error types
pub mod error;

// Domain-based modules (GAP-001.b)
pub mod disk;
pub mod encryption;
pub mod filesystem;
pub mod gpt;
pub mod image;
pub mod lvm;
pub mod manager;
pub mod partition;
pub mod smart;
pub mod util;
pub mod volume;

// Re-export storage-common types (canonical domain models)
pub use storage_common;
pub use storage_common::ProcessInfo;

// Re-export format utilities (now in storage-common)
pub use storage_common::{bytes_to_pretty, get_numeric, get_step, pretty_to_bytes};

// Re-export partition type catalog (now in storage-common)
pub use storage_common::{
    COMMON_DOS_TYPES, COMMON_GPT_TYPES, PartitionTypeInfo, PartitionTypeInfoFlags,
    get_all_partition_type_infos, get_valid_partition_names, make_partition_flags_bits,
};

// Re-export volume types (now in storage-common)
pub use storage_common::{VolumeKind, VolumeType};

// Re-export CreatePartitionInfo (now in storage-common)
pub use storage_common::CreatePartitionInfo;

// Re-export GPT alignment (now in storage-common)
pub use storage_common::GPT_ALIGNMENT_BYTES;

// Re-export commonly used zbus types
pub use zbus::zvariant::OwnedObjectPath;

// Explicit exports from dbus module (DBus byte string encoding/decoding)
pub use dbus::bytestring::{
    bytestring_owned_value, decode_c_string_bytes, decode_mount_points, encode_bytestring,
    owned_value_to_bytestring,
};

// Re-export key types from new modules
// Discovery and operations return storage_common types only
pub use manager::{DeviceEvent, DeviceEventStream, DiskManager};
pub use smart::{SmartInfo, SmartSelfTestKind};

// Re-export error types
pub use error::DiskError;

// Re-export configuration types
pub use encryption::config::EncryptionOptionsSettings;
pub use filesystem::MountOptionsSettings;

// Re-export operations from new domain modules
pub use disk::{
    device_apis::{loop_setup_device_path, open_for_backup_by_device, open_for_restore_by_device},
    discovery::{
        block_object_path_for_device, get_disk_info_for_drive_path, get_disks,
        get_disks_with_partitions, get_disks_with_volumes,
    },
    format::format_disk,
    image::{open_for_backup, open_for_restore},
    power::{
        eject_drive, eject_drive_by_device, power_off_drive, power_off_drive_by_device,
        remove_drive, remove_drive_by_device, standby_drive, standby_drive_by_device, wakeup_drive,
        wakeup_drive_by_device,
    },
};
pub use gpt::{fallback_gpt_usable_range_bytes, probe_gpt_usable_range_bytes};
pub use lvm::list_lvs_for_pv;
pub use util::{find_processes_using_mount, kill_processes};

// Partition operations (from new partition module)
pub use partition::{
    create_partition, create_partition_table, create_partition_with_filesystem, delete_partition,
    edit_partition, resize_partition, set_partition_flags, set_partition_name, set_partition_type,
};

// Filesystem operations (from new filesystem module)
pub use filesystem::{
    check_filesystem, format_filesystem, get_filesystem_label, get_mount_options, get_mount_point,
    mount_filesystem, repair_filesystem, reset_mount_options, set_filesystem_label,
    set_mount_options, take_filesystem_ownership, unmount_filesystem,
};

// Encryption operations (from new encryption module)
pub use encryption::config::{
    clear_encryption_options, get_encryption_options, set_encryption_options,
};
pub use encryption::{
    change_luks_passphrase, format_luks, list_luks_devices, lock_luks, unlock_luks,
};

// SMART operations (from new smart module)
pub use smart::{
    abort_drive_smart_selftest, get_drive_smart_info, get_smart_info_by_device,
    start_drive_smart_selftest, start_drive_smart_selftest_by_device,
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
