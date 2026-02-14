// SPDX-License-Identifier: GPL-3.0-only

//! High-level UDisks2 operation functions
//! 
//! This module provides clean, testable wrappers around UDisks2 D-Bus operations.
//! These functions encapsulate all the proxy building and error handling,
//! allowing storage-service to focus on authorization and orchestration.

pub mod partitions;
pub mod filesystems;
pub mod luks;

pub use partitions::{
    create_partition_table,
    create_partition,
    delete_partition,
    resize_partition,
    set_partition_type,
    set_partition_flags,
    set_partition_name,
};

pub use filesystems::{
    format_filesystem,
    mount_filesystem,
    unmount_filesystem,
    check_filesystem,
    set_filesystem_label,
    get_filesystem_label,
    take_filesystem_ownership,
    get_mount_point,
};

pub use luks::{
    unlock_luks,
    lock_luks,
    change_luks_passphrase,
    format_luks,
    list_luks_devices,
};
