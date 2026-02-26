// SPDX-License-Identifier: GPL-3.0-only

pub mod protocol;
pub mod traits;

pub use protocol::{
    OperationEvent, OperationId, OperationKind, OperationProgress, StorageError, StorageErrorKind,
};
pub use traits::{
    DiskDiscovery, DiskOpsAdapter, DiskQueryAdapter, FilesystemDiscovery, FilesystemOpsAdapter,
    ImageOpsAdapter, LuksOpsAdapter, PartitionOpsAdapter, Partitioning,
};
