mod dbus;
mod disks;
mod operations;
mod options;
mod udisks_block_config;
mod usage;

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

// Explicit exports from disks module
pub use disks::{
    BtrfsFilesystem, BtrfsSubvolume, DeviceEvent, DeviceEventStream,
    DiskError, DiskManager, DriveModel, EncryptionOptionsSettings,
    MountOptionsSettings, SmartInfo, SmartSelfTestKind,
    VolumeModel, VolumeNode, fallback_gpt_usable_range_bytes,
    find_processes_using_mount, kill_processes, list_lvs_for_pv,
    probe_gpt_usable_range_bytes,
};

// Explicit exports from disks::image submodule
pub use disks::image::{loop_setup, mount_filesystem, open_for_backup, open_for_restore};

// NOTE: format utilities moved to storage-models (already re-exported above)

// Explicit exports from options module (mount/encryption option parsing)
pub use options::{
    join_options, merge_other_with_managed, normalize_options, remove_prefixed, remove_token,
    set_prefixed_value, set_token_present, split_options, stable_dedup,
};

// NOTE: partition type catalog moved to storage-models (already re-exported above)

// Explicit exports from udisks_block_config module (UDisks2 configuration helpers)
pub use udisks_block_config::{ConfigurationItem, UDisks2BlockConfigurationProxy};

// Explicit exports from usage module (filesystem usage statistics)
pub use usage::{Usage, usage_for_mount_point};

// Export high-level operation functions
pub use operations::{
    // Partition operations
    create_partition_table,
    create_partition,
    delete_partition,
    resize_partition,
    set_partition_type,
    set_partition_flags,
    set_partition_name,
    // Filesystem operations
    format_filesystem,
    mount_filesystem as mount_filesystem_op,
    unmount_filesystem,
    check_filesystem,
    set_filesystem_label,
    get_filesystem_label,
    take_filesystem_ownership,
    get_mount_point,
    // LUKS operations
    unlock_luks,
    lock_luks,
    change_luks_passphrase,
    format_luks,
    list_luks_devices,
};
