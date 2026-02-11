use crate::config::Config;
use crate::ui::app::state::ContextPage;
use crate::ui::dialogs::message::{
    AttachDiskImageDialogMessage, FormatDiskMessage, ImageOperationDialogMessage,
    NewDiskImageDialogMessage, SmartDialogMessage, UnmountBusyMessage,
};
use crate::ui::dialogs::state::ShowDialog;
use crate::ui::volumes::VolumesControlMessage;
use disks_dbus::DriveModel;

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
    UpdateNav(Vec<DriveModel>, Option<String>),
    UpdateNavWithChildSelection(Vec<DriveModel>, Option<String>),
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
        object_path: String,
    },
    SidebarClearChildSelection,
    SidebarToggleExpanded(crate::ui::sidebar::SidebarNodeKey),
    SidebarDriveEject(String),
    SidebarVolumeUnmount {
        drive: String,
        object_path: String,
    },
    SmartDialog(SmartDialogMessage),
    NewDiskImage,
    AttachDisk,
    #[allow(dead_code)]
    CreateDiskFrom,
    #[allow(dead_code)]
    RestoreImageTo,
    #[allow(dead_code)]
    CreateDiskFromPartition,
    #[allow(dead_code)]
    RestoreImageToPartition,
    NewDiskImageDialog(NewDiskImageDialogMessage),
    AttachDiskImageDialog(AttachDiskImageDialogMessage),
    ImageOperationDialog(ImageOperationDialogMessage),
    UnmountBusy(UnmountBusyMessage),
    RetryUnmountAfterKill(String),
    OpenImagePathPicker(ImagePathPickerKind),
    ImagePathPicked(ImagePathPickerKind, Option<String>),
    ToggleShowReserved(bool),
    #[allow(dead_code)]
    Surface(cosmic::surface::Action),
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
