// SPDX-License-Identifier: GPL-3.0-only

use storage_types::UsageScanParallelismPreset;

pub(crate) fn map_parallelism_threads(
    preset: UsageScanParallelismPreset,
    cpu_count: usize,
) -> usize {
    let cpus = cpu_count.max(1);
    match preset {
        UsageScanParallelismPreset::Low => cpus.div_ceil(4).max(1),
        UsageScanParallelismPreset::Balanced => cpus.div_ceil(2).max(1),
        UsageScanParallelismPreset::High => cpus,
    }
}
