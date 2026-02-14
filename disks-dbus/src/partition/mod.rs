//! Partition operations
//!
//! This module provides operations for managing partitions:
//! - Creating and deleting partitions
//! - Resizing partitions
//! - Editing partition properties (type, name, flags)

mod create;
mod delete;
mod resize;
mod edit;
mod info;

pub use create::{create_partition_table, create_partition};
pub use delete::delete_partition;
pub use resize::resize_partition;
pub use edit::{set_partition_type, set_partition_flags, set_partition_name, edit_partition};
pub use info::make_partition_flags_bits;

