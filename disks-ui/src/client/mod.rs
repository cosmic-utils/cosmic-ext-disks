// SPDX-License-Identifier: GPL-3.0-only

//! D-Bus client wrappers for storage-service operations

pub mod btrfs;
pub mod error;

pub use btrfs::BtrfsClient;
pub use error::ClientError;
