/// Flags describing a partition type.
#[derive(Debug, Clone, Copy, Default)]
pub enum PartitionTypeInfoFlags {
    /// No flags set.
    #[default]
    None = 0,
    /// Partition type is used for swap.
    Swap = (1 << 0),
    /// Partition type is used for RAID/LVM or similar.
    Raid = (1 << 1),
    /// Partition type indicates the partition is hidden
    /// (e.g. 'dos' type 0x1b PartitionType::new("Hidden W95 FAT32").
    /// Note that this is not the same as user-toggleable
    /// attributes/flags for a partition.
    Hidden = (1 << 2),
    /// Partition type can only be used when creating a partition
    /// and e.g. should not be selectable in a "change partition type"
    /// user interface (e.g. 'dos' type 0x05, 0x0f and 0x85
    /// for extended partitions).
    CreateOnly = (1 << 3),
    /// Partition type indicates the partition is part of the system / bootloader (e.g. 'dos' types 0xee, 0xff, 'gpt' types for 'EFI System partition' and 'BIOS Boot partition').
    System = (1 << 4),
}

/// Detailed information about a partition type.
///
/// `table_subtype` is used to break the set of partition types for
/// `table_type` into a logical subsets. It is typically only used in
/// user interfaces where the partition type is selected.
#[derive(Debug, Clone, Copy, Default)]
pub struct PartitionTypeInfo {
    /// A partition table type e.g. `dos` or `gpt`
    pub table_type: &'static str,
    /// A partition table sub-type
    pub table_subtype: &'static str,
    /// A partition type
    pub ty: &'static str,
    /// Name of the partition
    pub name: &'static str,
    /// Flags describing the partition type
    pub flags: PartitionTypeInfoFlags,
    /// Default filesystem type for this partition type
    pub filesystem_type: &'static str,
}

impl PartitionTypeInfo {
    const fn new(
        table_type: &'static str,
        table_subtype: &'static str,
        ty: &'static str,
        name: &'static str,
        flags: PartitionTypeInfoFlags,
        filesystem_type: &'static str,
    ) -> Self {
        //TODO: wrap name with gettext call
        Self {
            table_type,
            table_subtype,
            ty,
            name,
            flags,
            filesystem_type,
        }
    }

    pub fn find_by_id(type_id: String) -> Option<PartitionTypeInfo> {
        PARTITION_TYPES.iter().find(|p| p.ty == type_id).cloned()
    }
}

pub fn get_valid_partition_names(table_type: String) -> Vec<String> {
    match table_type.as_str() {
        "gpt" => COMMON_GPT_TYPES
            .iter()
            .map(|p| format!("{} - {}", p.name, p.ty))
            .collect(),
        "dos" => COMMON_DOS_TYPES
            .iter()
            .map(|p| format!("{} - {}", p.name, p.ty))
            .collect(),
        _ => vec![],
    }
}

pub fn get_all_partition_type_infos(table_type: &str) -> Vec<PartitionTypeInfo> {
    PARTITION_TYPES
        .iter()
        .filter(|p| p.table_type == table_type)
        .cloned()
        .collect()
}

mod apm;
mod dos;
mod gpt;

pub use dos::COMMON_DOS_TYPES;
pub use gpt::COMMON_GPT_TYPES;

const GPT_LEN: usize = gpt::PARTITION_TYPES.len();
const DOS_LEN: usize = dos::PARTITION_TYPES.len();
const APM_LEN: usize = apm::PARTITION_TYPES.len();

const EMPTY_PARTITION_TYPE: PartitionTypeInfo =
    PartitionTypeInfo::new("", "", "", "", PartitionTypeInfoFlags::None, "");

const fn concat_partition_types() -> [PartitionTypeInfo; GPT_LEN + DOS_LEN + APM_LEN] {
    let mut out = [EMPTY_PARTITION_TYPE; GPT_LEN + DOS_LEN + APM_LEN];
    let mut i = 0;
    while i < GPT_LEN {
        out[i] = gpt::PARTITION_TYPES[i];
        i += 1;
    }
    let mut j = 0;
    while j < DOS_LEN {
        out[GPT_LEN + j] = dos::PARTITION_TYPES[j];
        j += 1;
    }
    let mut k = 0;
    while k < APM_LEN {
        out[GPT_LEN + DOS_LEN + k] = apm::PARTITION_TYPES[k];
        k += 1;
    }
    out
}

/// Known [PartitionType]s.
pub static PARTITION_TYPES: [PartitionTypeInfo; GPT_LEN + DOS_LEN + APM_LEN] =
    concat_partition_types();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_type_catalog_count_is_stable() {
        assert_eq!(PARTITION_TYPES.len(), 228);
        assert_eq!(GPT_LEN, 178);
        assert_eq!(DOS_LEN, 37);
        assert_eq!(APM_LEN, 13);
    }

    #[test]
    fn partition_type_catalog_contains_known_ids() {
        let efi = PartitionTypeInfo::find_by_id("c12a7328-f81f-11d2-ba4b-00a0c93ec93b".to_string())
            .expect("EFI System partition type must exist");
        assert_eq!(efi.table_type, "gpt");

        let apm_map = PartitionTypeInfo::find_by_id("Apple_partition_map".to_string())
            .expect("APM partition map type must exist");
        assert_eq!(apm_map.table_type, "apm");
    }
}
