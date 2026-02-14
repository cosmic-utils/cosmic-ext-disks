mod dbus;
mod options;
mod udisks_block_config;
mod usage;

// Error types
pub mod error;

// Domain-based modules (GAP-001.b)
pub mod disk;
pub mod partition;
pub mod filesystem;
pub mod encryption;
pub mod image;
pub mod smart;
pub mod lvm;
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
    get_all_partition_type_infos, get_valid_partition_names, make_partition_flags_bits,
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
// Discovery and operations return storage_models types only
pub use manager::{DiskManager, DeviceEvent, DeviceEventStream};
pub use smart::{SmartInfo, SmartSelfTestKind};

// Re-export error types
pub use error::DiskError;

// Re-export configuration types  
pub use filesystem::MountOptionsSettings;
pub use encryption::config::EncryptionOptionsSettings;

// Re-export operations from new domain modules
pub use gpt::{fallback_gpt_usable_range_bytes, probe_gpt_usable_range_bytes};
pub use lvm::list_lvs_for_pv;
pub use util::{find_processes_using_mount, kill_processes};
pub use disk::{
    discovery::{
        block_object_path_for_device, get_disk_info_for_drive_path, get_disks,
        get_disks_with_partitions, get_disks_with_volumes,
    },
    power::{
        eject_drive, eject_drive_by_device, power_off_drive, power_off_drive_by_device, remove_drive,
        remove_drive_by_device, standby_drive, standby_drive_by_device, wakeup_drive, wakeup_drive_by_device,
    },
    format::format_disk,
    image::{open_for_backup, open_for_restore},
    device_apis::{open_for_backup_by_device, open_for_restore_by_device, loop_setup_device_path},
};

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
    get_mount_options, set_mount_options, reset_mount_options,
};

// Encryption operations (from new encryption module)
pub use encryption::{
    unlock_luks, lock_luks, change_luks_passphrase,
    format_luks, list_luks_devices,
};
pub use encryption::config::{
    get_encryption_options, set_encryption_options, clear_encryption_options,
};

// SMART operations (from new smart module)
pub use smart::{
    get_drive_smart_info, get_smart_info_by_device, start_drive_smart_selftest,
    start_drive_smart_selftest_by_device, abort_drive_smart_selftest,
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
