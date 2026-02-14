mod dbus;
mod disks;
mod format;
mod options;
mod partition_types;
mod udisks_block_config;
mod usage;

// Re-export storage-models types (canonical domain models)
pub use storage_models;
pub use storage_models::ProcessInfo;

// Re-export commonly used zbus types
pub use zbus::zvariant::OwnedObjectPath;

// Explicit exports from dbus module (DBus byte string encoding/decoding)
pub use dbus::bytestring::{
    bytestring_owned_value, decode_c_string_bytes, decode_mount_points, encode_bytestring,
    owned_value_to_bytestring,
};

// Explicit exports from disks module
pub use disks::{
    BtrfsFilesystem, BtrfsSubvolume, CreatePartitionInfo, DeviceEvent, DeviceEventStream,
    DiskError, DiskManager, DriveModel, EncryptionOptionsSettings, GPT_ALIGNMENT_BYTES,
    MountOptionsSettings, SmartInfo, SmartSelfTestKind, VolumeKind,
    VolumeModel, VolumeNode, VolumeType, fallback_gpt_usable_range_bytes,
    find_processes_using_mount, kill_processes, list_lvs_for_pv, loop_setup, mount_filesystem,
    probe_gpt_usable_range_bytes,
};

// Explicit exports from format module
pub use format::{bytes_to_pretty, get_numeric, get_step, pretty_to_bytes};

// Explicit exports from options module (mount/encryption option parsing)
pub use options::{
    join_options, merge_other_with_managed, normalize_options, remove_prefixed, remove_token,
    set_prefixed_value, set_token_present, split_options, stable_dedup,
};

// Explicit exports from partition_types module (partition type catalogs)
pub use partition_types::{
    COMMON_DOS_TYPES, COMMON_GPT_TYPES, PartitionTypeInfo, PartitionTypeInfoFlags,
    get_all_partition_type_infos, get_valid_partition_names,
};

// Explicit exports from udisks_block_config module (UDisks2 configuration helpers)
pub use udisks_block_config::{ConfigurationItem, UDisks2BlockConfigurationProxy};

// Explicit exports from usage module (filesystem usage statistics)
pub use usage::{Usage, usage_for_mount_point};
