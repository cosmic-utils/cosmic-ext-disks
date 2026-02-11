use serde::Deserialize;

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

impl<'de> Deserialize<'de> for PartitionTypeInfoFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "" => Ok(PartitionTypeInfoFlags::None),
            "None" => Ok(PartitionTypeInfoFlags::None),
            "Swap" => Ok(PartitionTypeInfoFlags::Swap),
            "Raid" => Ok(PartitionTypeInfoFlags::Raid),
            "Hidden" => Ok(PartitionTypeInfoFlags::Hidden),
            "CreateOnly" => Ok(PartitionTypeInfoFlags::CreateOnly),
            "System" => Ok(PartitionTypeInfoFlags::System),
            _ => Err(serde::de::Error::custom(format!("Unknown flag: {}", s))),
        }
    }
}

/// Detailed information about a partition type.
///
/// `table_subtype` is used to break the set of partition types for
/// `table_type` into a logical subsets. It is typically only used in
/// user interfaces where the partition type is selected.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PartitionTypeInfo {
    /// A partition table type e.g. `dos` or `gpt`
    pub table_type: String,
    /// A partition table sub-type
    pub table_subtype: String,
    /// A partition type
    pub ty: String,
    /// Name of the partition
    pub name: String,
    /// Flags describing the partition type
    pub flags: PartitionTypeInfoFlags,
    /// Default filesystem type for this partition type
    pub filesystem_type: String,
}

impl PartitionTypeInfo {
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

// Load TOML data at compile time
const GPT_TOML: &str = include_str!("../data/gpt_types.toml");
const DOS_TOML: &str = include_str!("../data/dos_types.toml");
const APM_TOML: &str = include_str!("../data/apm_types.toml");
const COMMON_GPT_TOML: &str = include_str!("../data/common_gpt_types.toml");
const COMMON_DOS_TOML: &str = include_str!("../data/common_dos_types.toml");

#[derive(Deserialize)]
struct PartitionTypeCatalog {
    types: Vec<PartitionTypeInfo>,
}

// Lazy-loaded parsed data
static PARTITION_TYPES_DATA: std::sync::LazyLock<Vec<PartitionTypeInfo>> =
    std::sync::LazyLock::new(|| {
        let mut all_types = Vec::new();

        // Parse GPT types
        if let Ok(gpt) = toml::from_str::<PartitionTypeCatalog>(GPT_TOML) {
            all_types.extend(gpt.types);
        }

        // Parse DOS types
        if let Ok(dos) = toml::from_str::<PartitionTypeCatalog>(DOS_TOML) {
            all_types.extend(dos.types);
        }

        // Parse APM types
        if let Ok(apm) = toml::from_str::<PartitionTypeCatalog>(APM_TOML) {
            all_types.extend(apm.types);
        }

        all_types
    });

/// Known [PartitionType]s.
pub static PARTITION_TYPES: std::sync::LazyLock<Vec<PartitionTypeInfo>> =
    std::sync::LazyLock::new(|| PARTITION_TYPES_DATA.clone());

/// Common GPT partition types for UI display (user-selectable filesystem types)
/// Loaded from dedicated common_gpt_types.toml containing exactly 10 filesystem types
/// as specified in the partitioning view improvement spec
pub static COMMON_GPT_TYPES: std::sync::LazyLock<Vec<PartitionTypeInfo>> =
    std::sync::LazyLock::new(|| {
        if let Ok(catalog) = toml::from_str::<PartitionTypeCatalog>(COMMON_GPT_TOML) {
            catalog.types
        } else {
            vec![]
        }
    });

/// Common DOS partition types for UI display (user-selectable filesystem types)
/// Loaded from dedicated common_dos_types.toml containing exactly 10 filesystem types
/// as specified in the partitioning view improvement spec
pub static COMMON_DOS_TYPES: std::sync::LazyLock<Vec<PartitionTypeInfo>> =
    std::sync::LazyLock::new(|| {
        if let Ok(catalog) = toml::from_str::<PartitionTypeCatalog>(COMMON_DOS_TOML) {
            catalog.types
        } else {
            vec![]
        }
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_type_catalog_count_is_stable() {
        // We now load from TOML, so total is 249 (189 GPT + 47 DOS + 13 APM)
        // Added: Btrfs, F2FS, UDF to both GPT and DOS; XFS to DOS
        assert_eq!(PARTITION_TYPES.len(), 249);

        let gpt_count = PARTITION_TYPES
            .iter()
            .filter(|p| p.table_type == "gpt")
            .count();
        let dos_count = PARTITION_TYPES
            .iter()
            .filter(|p| p.table_type == "dos")
            .count();
        let apm_count = PARTITION_TYPES
            .iter()
            .filter(|p| p.table_type == "apm")
            .count();

        assert_eq!(gpt_count, 189);
        assert_eq!(dos_count, 47);
        assert_eq!(apm_count, 13);
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

    #[test]
    fn common_types_have_filesystem_types() {
        // All common types should have non-empty filesystem_type
        // This ensures no labelless radio buttons appear in the UI
        for p in COMMON_GPT_TYPES.iter() {
            assert!(
                !p.filesystem_type.is_empty(),
                "GPT partition type '{}' has empty filesystem_type",
                p.name
            );
        }

        for p in COMMON_DOS_TYPES.iter() {
            assert!(
                !p.filesystem_type.is_empty(),
                "DOS partition type '{}' has empty filesystem_type",
                p.name
            );
        }

        // Verify expected count (spec defines 10-11 filesystem types)
        // GPT: ext4, ext3, xfs, btrfs, f2fs, udf, ntfs, vfat, exfat, swap = 10
        // DOS: same subset = 10
        assert_eq!(
            COMMON_GPT_TYPES.len(),
            10,
            "Expected 10 common GPT filesystem types"
        );
        assert_eq!(
            COMMON_DOS_TYPES.len(),
            10,
            "Expected 10 common DOS filesystem types"
        );
    }
}
