use std::sync::Arc;

use async_trait::async_trait;

use storage_testing::errors::Result;

pub mod btrfs;
pub mod common;
pub mod disk;
pub mod filesystem;
pub mod image;
pub mod logical;
pub mod luks;
pub mod partition;
pub mod rclone;

#[derive(Debug, Clone, Default)]
pub struct HarnessContext {
    pub dry_run: bool,
}

#[async_trait]
pub trait HarnessTest: Send + Sync {
    fn id(&self) -> &'static str;
    fn suite(&self) -> &'static str;
    fn required_spec(&self) -> &'static str;
    fn exclusive(&self) -> bool {
        false
    }

    async fn execute(&self, ctx: &HarnessContext) -> Result<()>;
}

pub type TestRef = Arc<dyn HarnessTest>;

pub fn instantiate_tests() -> Vec<TestRef> {
    vec![
        Arc::new(disk::list_disks::DiskListDisks),
        Arc::new(disk::list_volumes_schema_integrity::DiskListVolumesSchemaIntegrity),
        Arc::new(disk::get_disk_info_for_known_device::DiskGetDiskInfoForKnownDevice),
        Arc::new(filesystem::unmount_roundtrip::FilesystemMountUnmountRoundtrip),
        Arc::new(filesystem::check_readonly_path::FilesystemCheckReadonlyPath),
        Arc::new(filesystem::usage_scan_basic::FilesystemUsageScanBasic),
        Arc::new(filesystem::mount_options_roundtrip::FilesystemMountOptionsReadWriteRoundtrip),
        Arc::new(partition::list_partitions_expected_from_spec::PartitionListPartitionsExpectedFromSpec),
        Arc::new(partition::create_delete_roundtrip::PartitionCreateDeleteRoundtrip),
        Arc::new(partition::set_name_type_flags_roundtrip::PartitionSetNameTypeFlagsRoundtrip),
        Arc::new(luks::unlock_lock_roundtrip::LuksUnlockLockRoundtrip),
        Arc::new(luks::options_roundtrip::LuksOptionsReadWriteRoundtrip),
        Arc::new(btrfs::subvolume_create_delete_roundtrip::BtrfsSubvolumeCreateDeleteRoundtrip),
        Arc::new(btrfs::snapshot_create_delete_roundtrip::BtrfsSnapshotCreateDeleteRoundtrip),
        Arc::new(btrfs::default_subvolume_set_get::BtrfsDefaultSubvolumeSetGet),
        Arc::new(logical::list_entities_schema_integrity::LogicalListEntitiesSchemaIntegrity),
        Arc::new(logical::lvm_create_resize_delete_lv::LogicalLvmCreateResizeDeleteLv),
        Arc::new(logical::mdraid_create_start_stop_delete::LogicalMdraidCreateStartStopDelete),
        Arc::new(logical::btrfs_add_remove_member::LogicalBtrfsAddRemoveMember),
        Arc::new(image::loop_setup_valid_image::ImageLoopSetupValidImage),
        Arc::new(image::backup_restore_drive_smoke::ImageBackupRestoreDriveSmoke),
        Arc::new(rclone::list_remotes_basic::RcloneListRemotesBasic),
        Arc::new(rclone::mount_status_query::RcloneMountStatusQuery),
    ]
}
