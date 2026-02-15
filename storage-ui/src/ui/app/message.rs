use crate::config::Config;
use crate::models::UiDrive;
use crate::ui::app::state::ContextPage;
use crate::ui::dialogs::message::{
    AttachDiskImageDialogMessage, FormatDiskMessage, ImageOperationDialogMessage,
    NewDiskImageDialogMessage, SmartDialogMessage, UnmountBusyMessage,
};
use crate::ui::dialogs::state::ShowDialog;
use crate::ui::volumes::VolumesControlMessage;

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Message {
    OpenRepositoryUrl,
    OpenPath(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    LaunchUrl(String),
    VolumesMessage(VolumesControlMessage),
    FormatDisk(FormatDiskMessage),
    DriveRemoved(String),
    DriveAdded(String),
    None,
    UpdateNav(Vec<UiDrive>, Option<String>),
    UpdateNavWithChildSelection(Vec<UiDrive>, Option<String>),
    Dialog(Box<ShowDialog>),
    CloseDialog,
    Eject,
    PowerOff,
    Format,
    SmartData,
    StandbyNow,
    Wakeup,

    // Sidebar (custom treeview)
    SidebarSelectDrive(String),
    SidebarSelectChild {
        device_path: String,
    },
    SidebarClearChildSelection,
    SidebarToggleExpanded(crate::ui::sidebar::SidebarNodeKey),
    SidebarDriveEject(String),
    SidebarVolumeUnmount {
        drive: String,
        device_path: String,
    },
    SmartDialog(SmartDialogMessage),
    NewDiskImage,
    AttachDisk,
    CreateDiskFrom,
    RestoreImageTo,
    CreateDiskFromPartition,
    RestoreImageToPartition,
    NewDiskImageDialog(NewDiskImageDialogMessage),
    AttachDiskImageDialog(AttachDiskImageDialogMessage),
    ImageOperationDialog(ImageOperationDialogMessage),
    /// Emitted when Phase 1 completes; store operation_id and start progress subscription.
    ImageOperationStarted(String),
    UnmountBusy(UnmountBusyMessage),
    RetryUnmountAfterKill(String),
    OpenImagePathPicker(ImagePathPickerKind),
    ImagePathPicked(ImagePathPickerKind, Option<String>),
    ToggleShowReserved(bool),

    // BTRFS management
    BtrfsLoadSubvolumes {
        block_path: String,
        mount_point: String,
    },
    BtrfsSubvolumesLoaded {
        mount_point: String,
        result: Result<Vec<storage_common::BtrfsSubvolume>, String>,
    },
    BtrfsDeleteSubvolume {
        block_path: String,
        mount_point: String,
        path: String,
    },
    BtrfsDeleteSubvolumeConfirm {
        block_path: String,
        mount_point: String,
        path: String,
    },
    BtrfsLoadUsage {
        block_path: String,
        mount_point: String,
    },
    BtrfsUsageLoaded {
        mount_point: String,
        used_space: Result<u64, String>,
    },
    BtrfsToggleSubvolumeExpanded {
        mount_point: String,
        subvolume_id: u64,
    },
    BtrfsLoadDefaultSubvolume {
        mount_point: String,
    },
    BtrfsDefaultSubvolumeLoaded {
        mount_point: String,
        result: Result<storage_common::BtrfsSubvolume, String>,
    },
    BtrfsSetDefaultSubvolume {
        mount_point: String,
        subvolume_id: u64,
    },
    BtrfsToggleReadonly {
        mount_point: String,
        subvolume_id: u64,
    },
    BtrfsReadonlyToggled {
        mount_point: String,
        result: Result<(), String>,
    },
    BtrfsShowProperties {
        mount_point: String,
        subvolume_id: u64,
    },
    BtrfsCloseProperties {
        mount_point: String,
    },
    BtrfsLoadDeletedSubvolumes {
        mount_point: String,
    },
    BtrfsDeletedSubvolumesLoaded {
        mount_point: String,
        result: Result<Vec<storage_common::DeletedSubvolume>, String>,
    },
    BtrfsToggleShowDeleted {
        mount_point: String,
    },
    BtrfsRefreshAll {
        mount_point: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImagePathPickerKind {
    NewDiskImage,
    AttachDiskImage,
    ImageOperationCreate,
    ImageOperationRestore,
}

impl From<FormatDiskMessage> for Message {
    fn from(val: FormatDiskMessage) -> Self {
        Message::FormatDisk(val)
    }
}

impl From<SmartDialogMessage> for Message {
    fn from(val: SmartDialogMessage) -> Self {
        Message::SmartDialog(val)
    }
}

impl From<NewDiskImageDialogMessage> for Message {
    fn from(val: NewDiskImageDialogMessage) -> Self {
        Message::NewDiskImageDialog(val)
    }
}

impl From<AttachDiskImageDialogMessage> for Message {
    fn from(val: AttachDiskImageDialogMessage) -> Self {
        Message::AttachDiskImageDialog(val)
    }
}

impl From<ImageOperationDialogMessage> for Message {
    fn from(val: ImageOperationDialogMessage) -> Self {
        Message::ImageOperationDialog(val)
    }
}

impl From<UnmountBusyMessage> for Message {
    fn from(val: UnmountBusyMessage) -> Self {
        Message::UnmountBusy(val)
    }
}
