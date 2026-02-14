//! SMART monitoring operations
//!
//! This module provides operations for disk health monitoring via SMART:
//! - Reading SMART data
//! - Running self-tests
//! - SMART data types

pub mod types;
pub mod info;
pub mod test;

pub use types::*;
pub use info::{get_drive_smart_info, get_smart_info_by_device};
pub use test::{start_drive_smart_selftest, start_drive_smart_selftest_by_device, abort_drive_smart_selftest};
