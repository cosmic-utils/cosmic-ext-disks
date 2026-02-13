#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditPartitionMessage {
    TypeUpdate(usize),
    NameUpdate(String),
    LegacyBiosBootableUpdate(bool),
    SystemPartitionUpdate(bool),
    HiddenUpdate(bool),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResizePartitionMessage {
    SizeUpdate(u64),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditFilesystemLabelMessage {
    LabelUpdate(String),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditMountOptionsMessage {
    UseDefaultsUpdate(bool),
    MountAtStartupUpdate(bool),
    RequireAuthUpdate(bool),
    ShowInUiUpdate(bool),
    OtherOptionsUpdate(String),
    DisplayNameUpdate(String),
    IconNameUpdate(String),
    SymbolicIconNameUpdate(String),
    MountPointUpdate(String),
    IdentifyAsIndexUpdate(usize),
    FilesystemTypeUpdate(String),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TakeOwnershipMessage {
    RecursiveUpdate(bool),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangePassphraseMessage {
    CurrentUpdate(String),
    NewUpdate(String),
    ConfirmUpdate(String),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditEncryptionOptionsMessage {
    UseDefaultsUpdate(bool),
    UnlockAtStartupUpdate(bool),
    RequireAuthUpdate(bool),
    OtherOptionsUpdate(String),
    NameUpdate(String),
    PassphraseUpdate(String),
    ShowPassphraseUpdate(bool),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateMessage {
    SizeUpdate(u64),
    SizeUnitUpdate(usize),
    NameUpdate(String),
    PasswordUpdate(String),
    ConfirmedPasswordUpdate(String),
    PasswordProtectedUpdate(bool),
    EraseUpdate(bool),
    PartitionTypeUpdate(usize),
    Cancel,
    Partition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnlockMessage {
    PassphraseUpdate(String),
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatDiskMessage {
    EraseUpdate(usize),
    PartitioningUpdate(usize),
    Cancel,
    Confirm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmartDialogMessage {
    Refresh,
    SelfTestShort,
    SelfTestExtended,
    AbortSelfTest,
    Close,
    Loaded(Result<disks_dbus::SmartInfo, String>),
    ActionComplete(Result<(), String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NewDiskImageDialogMessage {
    SizeUpdate(u64),
    Create,
    Cancel,
    Complete(Result<(), String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachDiskImageDialogMessage {
    Attach,
    Cancel,
    Complete(Result<AttachDiskResult, String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachDiskResult {
    pub mounted: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageOperationDialogMessage {
    Start,
    CancelOperation,
    Complete(Result<(), String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnmountBusyMessage {
    Cancel,
    Retry,
    KillAndRetry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BtrfsCreateSubvolumeMessage {
    NameUpdate(String),
    Create,
    Cancel,
}
