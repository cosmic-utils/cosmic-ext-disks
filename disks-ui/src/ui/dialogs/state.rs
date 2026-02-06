use disks_dbus::{CreatePartitionInfo, DriveModel, PartitionTypeInfo, VolumeModel, VolumeNode};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ShowDialog {
    DeletePartition(DeletePartitionDialog),
    AddPartition(CreatePartitionDialog),
    FormatPartition(FormatPartitionDialog),
    EditPartition(EditPartitionDialog),
    ResizePartition(ResizePartitionDialog),
    EditFilesystemLabel(EditFilesystemLabelDialog),
    EditMountOptions(EditMountOptionsDialog),
    ConfirmAction(ConfirmActionDialog),
    TakeOwnership(TakeOwnershipDialog),
    ChangePassphrase(ChangePassphraseDialog),
    EditEncryptionOptions(EditEncryptionOptionsDialog),
    UnlockEncrypted(UnlockEncryptedDialog),
    FormatDisk(FormatDiskDialog),
    SmartData(SmartDataDialog),
    NewDiskImage(Box<NewDiskImageDialog>),
    AttachDiskImage(Box<AttachDiskImageDialog>),
    ImageOperation(Box<ImageOperationDialog>),
    Info { title: String, body: String },
}

#[derive(Debug, Clone)]
pub struct FormatPartitionDialog {
    pub volume: VolumeModel,
    pub info: CreatePartitionInfo,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct EditPartitionDialog {
    pub volume: VolumeModel,
    pub partition_types: Vec<PartitionTypeInfo>,
    pub selected_type_index: usize,
    pub name: String,
    pub legacy_bios_bootable: bool,
    pub system_partition: bool,
    pub hidden: bool,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct ResizePartitionDialog {
    pub volume: VolumeModel,
    pub min_size_bytes: u64,
    pub max_size_bytes: u64,
    pub new_size_bytes: u64,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub enum FilesystemTarget {
    Volume(VolumeModel),
    Node(VolumeNode),
}

#[derive(Debug, Clone)]
pub struct ConfirmActionDialog {
    pub title: String,
    pub body: String,
    pub target: FilesystemTarget,
    pub ok_message: crate::app::Message,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct EditFilesystemLabelDialog {
    pub target: FilesystemTarget,
    pub label: String,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct TakeOwnershipDialog {
    pub target: FilesystemTarget,
    pub recursive: bool,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct ChangePassphraseDialog {
    pub volume: VolumeModel,
    pub current_passphrase: String,
    pub new_passphrase: String,
    pub confirm_passphrase: String,
    pub error: Option<String>,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct EditMountOptionsDialog {
    pub target: FilesystemTarget,
    pub use_defaults: bool,
    pub mount_at_startup: bool,
    pub require_auth: bool,
    pub show_in_ui: bool,
    pub other_options: String,
    pub display_name: String,
    pub icon_name: String,
    pub symbolic_icon_name: String,
    pub mount_point: String,
    pub identify_as_options: Vec<String>,
    pub identify_as_index: usize,
    pub filesystem_type: String,
    pub error: Option<String>,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct EditEncryptionOptionsDialog {
    pub volume: VolumeModel,
    pub use_defaults: bool,
    pub unlock_at_startup: bool,
    pub require_auth: bool,
    pub other_options: String,
    pub name: String,
    pub passphrase: String,
    pub show_passphrase: bool,
    pub error: Option<String>,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct NewDiskImageDialog {
    pub path: String,
    pub size_bytes: u64,
    pub running: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttachDiskImageDialog {
    pub path: String,
    pub running: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageOperationKind {
    CreateFromDrive,
    RestoreToDrive,
    CreateFromPartition,
    RestoreToPartition,
}

#[derive(Debug, Clone)]
pub struct ImageOperationDialog {
    pub kind: ImageOperationKind,
    pub drive: DriveModel,
    pub partition: Option<VolumeModel>,
    pub image_path: String,
    pub running: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SmartDataDialog {
    pub drive: DriveModel,
    pub running: bool,
    pub info: Option<disks_dbus::SmartInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeletePartitionDialog {
    pub name: String,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct CreatePartitionDialog {
    pub info: CreatePartitionInfo,
    pub running: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FormatDiskDialog {
    pub drive: DriveModel,
    pub erase_index: usize,
    pub partitioning_index: usize,
    pub running: bool,
}

#[derive(Debug, Clone)]
pub struct UnlockEncryptedDialog {
    pub partition_path: String,
    pub partition_name: String,
    pub passphrase: String,
    pub error: Option<String>,
    pub running: bool,
}
