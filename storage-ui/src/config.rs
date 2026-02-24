// SPDX-License-Identifier: GPL-3.0-only

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use storage_common::UsageScanParallelismPreset;

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 2]
pub struct Config {
    demo: String,
    pub show_reserved: bool,
    pub usage_scan_parallelism: UsageScanParallelismPreset,
}
