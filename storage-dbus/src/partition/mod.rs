//! Partition operations
//!
//! This module provides operations for managing partitions:
//! - Creating and deleting partitions
//! - Resizing partitions
//! - Editing partition properties (type, name, flags)

mod create;
mod delete;
mod edit;
mod resize;

pub use create::{create_partition, create_partition_table};
pub use delete::delete_partition;
pub use edit::{edit_partition, set_partition_flags, set_partition_name, set_partition_type};
pub use resize::resize_partition;

// Re-export from storage-models
pub use storage_models::make_partition_flags_bits;
