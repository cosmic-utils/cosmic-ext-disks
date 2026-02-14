// SPDX-License-Identifier: GPL-3.0-only

//! Partition information and helper functions

use udisks2::partition::PartitionFlags;

/// Create a partition flags bitmask from individual boolean flags
/// 
/// This helper function converts individual boolean flags into a u64 bitmask
/// suitable for use with set_partition_flags.
pub fn make_partition_flags_bits(
    legacy_bios_bootable: bool,
    system_partition: bool,
    hidden: bool,
) -> u64 {
    let mut bits: u64 = 0;
    if system_partition {
        bits |= PartitionFlags::SystemPartition as u64;
    }
    if legacy_bios_bootable {
        bits |= PartitionFlags::LegacyBIOSBootable as u64;
    }
    if hidden {
        bits |= PartitionFlags::Hidden as u64;
    }
    bits
}
