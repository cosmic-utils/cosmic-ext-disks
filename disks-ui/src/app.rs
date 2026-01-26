// SPDX-License-Identifier: GPL-3.0-only

use crate::config::Config;
use crate::fl;
use crate::utils::{labelled_info, link_info};
use crate::views::about::about;
use crate::views::dialogs;
use crate::views::menu::{MenuAction, menu_view};
use crate::views::volumes::{VolumesControl, VolumesControlMessage};
use cosmic::app::{Core, Task, context_drawer};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Length, Subscription};
use cosmic::widget::text::heading;
use cosmic::widget::{self, Space, icon, menu, nav_bar};
use cosmic::{Application, ApplicationExt, Apply, Element, iced_widget};
use disks_dbus::bytes_to_pretty;
use disks_dbus::{DiskManager, DriveModel};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
pub const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    // Configuration data that persists between application runs.
    config: Config,

    image_op_cancel: Option<Arc<AtomicBool>>,

    pub dialog: Option<ShowDialog>,
}

pub use crate::ui::dialogs::message::{
    AttachDiskImageDialogMessage, AttachDiskResult, FormatDiskMessage, ImageOperationDialogMessage,
    NewDiskImageDialogMessage, SmartDialogMessage,
};
pub use crate::ui::dialogs::state::{
    AttachDiskImageDialog, ChangePassphraseDialog, ConfirmActionDialog, CreatePartitionDialog,
    DeletePartitionDialog, EditEncryptionOptionsDialog, EditFilesystemLabelDialog,
    EditMountOptionsDialog, EditPartitionDialog, FilesystemTarget, FormatDiskDialog,
    FormatPartitionDialog, ImageOperationDialog, ImageOperationKind, NewDiskImageDialog,
    ResizePartitionDialog, ShowDialog, SmartDataDialog, TakeOwnershipDialog, UnlockEncryptedDialog,
};

pub use crate::ui::app::message::Message;
pub use crate::ui::app::state::ContextPage;

/// Create a COSMIC application from the app model
impl Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.cosmos.Disks";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            nav: nav_bar::Model::default(),
            dialog: None,
            key_binds: HashMap::new(),
            image_op_cancel: None,
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        let nav_command = Task::perform(
            async {
                match DriveModel::get_drives().await {
                    Ok(drives) => Some(drives),
                    Err(e) => {
                        println!("Error: {e}");
                        None
                    }
                }
            },
            |drives| match drives {
                None => Message::None.into(),
                Some(drives) => Message::UpdateNav(drives, None).into(),
            },
        );

        (app, command.chain(nav_command))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        menu_view(&self.core, &self.key_binds)
    }

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        match self.dialog {
            Some(ref d) => match d {
                ShowDialog::DeletePartition(state) => Some(dialogs::confirmation(
                    fl!("delete", name = state.name.clone()),
                    fl!("delete-confirmation", name = state.name.clone()),
                    VolumesControlMessage::Delete.into(),
                    Some(Message::CloseDialog),
                    state.running,
                )),

                ShowDialog::AddPartition(state) => Some(dialogs::create_partition(state.clone())),

                ShowDialog::FormatPartition(state) => {
                    Some(dialogs::format_partition(state.clone()))
                }

                ShowDialog::EditPartition(state) => Some(dialogs::edit_partition(state.clone())),

                ShowDialog::ResizePartition(state) => {
                    Some(dialogs::resize_partition(state.clone()))
                }

                ShowDialog::EditFilesystemLabel(state) => {
                    Some(dialogs::edit_filesystem_label(state.clone()))
                }

                ShowDialog::EditMountOptions(state) => {
                    Some(dialogs::edit_mount_options(state.clone()))
                }

                ShowDialog::ConfirmAction(state) => Some(dialogs::confirmation(
                    state.title.clone(),
                    state.body.clone(),
                    state.ok_message.clone(),
                    Some(Message::CloseDialog),
                    state.running,
                )),

                ShowDialog::TakeOwnership(state) => Some(dialogs::take_ownership(state.clone())),

                ShowDialog::ChangePassphrase(state) => {
                    Some(dialogs::change_passphrase(state.clone()))
                }

                ShowDialog::EditEncryptionOptions(state) => {
                    Some(dialogs::edit_encryption_options(state.clone()))
                }

                ShowDialog::UnlockEncrypted(state) => {
                    Some(dialogs::unlock_encrypted(state.clone()))
                }

                ShowDialog::FormatDisk(state) => Some(dialogs::format_disk(state.clone())),

                ShowDialog::SmartData(state) => Some(dialogs::smart_data(state.clone())),

                ShowDialog::NewDiskImage(state) => {
                    Some(dialogs::new_disk_image(state.as_ref().clone()))
                }

                ShowDialog::AttachDiskImage(state) => {
                    Some(dialogs::attach_disk_image(state.as_ref().clone()))
                }

                ShowDialog::ImageOperation(state) => {
                    Some(dialogs::image_operation(state.as_ref().clone()))
                }

                ShowDialog::Info { title, body } => Some(dialogs::info(
                    title.clone(),
                    body.clone(),
                    Message::CloseDialog,
                )),
            },
            None => None,
        }
    }

    /// Allows overriding the default nav bar widget.
    fn nav_bar(&self) -> Option<Element<'_, cosmic::Action<Self::Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = widget::nav_bar(nav_model, |id| {
            cosmic::Action::Cosmic(cosmic::app::Action::NavBar(id))
        })
        .on_context(|id| cosmic::Action::Cosmic(cosmic::app::Action::NavBarContext(id)))
        // .context_menu(self.nav_context_menu(self.nav_bar()))
        .into_container()
        // XXX both must be shrink to avoid flex layout from ignoring it
        .width(cosmic::iced::Length::Shrink)
        .height(cosmic::iced::Length::Shrink);

        if !self.core().is_condensed() {
            nav = nav.max_width(280);
        }

        Some(Element::from(nav))
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        match self.nav.active_data::<DriveModel>() {
            None => widget::text::title1(fl!("no-disk-selected"))
                .apply(widget::container)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into(),

            Some(drive) => {
                let volumes_control = self.nav.active_data::<VolumesControl>().unwrap(); //TODO: Handle unwrap.

                let segment = volumes_control
                    .segments
                    .get(volumes_control.selected_segment)
                    .unwrap(); //TODO: Handle unwrap.
                let info = if let Some(v) = volumes_control.selected_volume_node() {
                    let mut col = iced_widget::column![
                        heading(v.label.clone()),
                        Space::new(0, 10),
                        labelled_info(fl!("size"), bytes_to_pretty(&v.size, true)),
                    ]
                    .spacing(5);

                    if let Some(usage) = &v.usage {
                        col = col.push(labelled_info(
                            fl!("usage"),
                            bytes_to_pretty(&usage.used, false),
                        ));
                    }

                    if let Some(mount_point) = v.mount_points.first() {
                        col = col.push(link_info(
                            fl!("mounted-at"),
                            mount_point,
                            Message::OpenPath(mount_point.clone()),
                        ));
                    }

                    let contents = if v.id_type.is_empty() {
                        match v.kind {
                            disks_dbus::VolumeKind::Filesystem => fl!("filesystem"),
                            disks_dbus::VolumeKind::LvmLogicalVolume => "LVM LV".to_string(),
                            disks_dbus::VolumeKind::LvmPhysicalVolume => "LVM PV".to_string(),
                            disks_dbus::VolumeKind::CryptoContainer => "LUKS".to_string(),
                            disks_dbus::VolumeKind::Partition => "Partition".to_string(),
                            disks_dbus::VolumeKind::Block => "Device".to_string(),
                        }
                    } else {
                        v.id_type.to_uppercase()
                    };

                    col.push(labelled_info(fl!("contents"), contents))
                        .push(labelled_info(
                            fl!("device"),
                            match v.device_path.as_ref() {
                                Some(s) => s.clone(),
                                None => fl!("unresolved"),
                            },
                        ))
                } else {
                    match segment.volume.clone() {
                        Some(p) => {
                            let mut name = p.name.clone();
                            if name.is_empty() {
                                name = fl!("partition-number", number = p.number);
                            } else {
                                name = fl!(
                                    "partition-number-with-name",
                                    number = p.number,
                                    name = name
                                );
                            }

                            let mut type_str = p.id_type.clone().to_uppercase();
                            type_str = format!("{} - {}", type_str, p.partition_type.clone());

                            let mut col = iced_widget::column![
                                heading(name),
                                Space::new(0, 10),
                                labelled_info(fl!("size"), bytes_to_pretty(&p.size, true)),
                            ]
                            .spacing(5);

                            if let Some(usage) = &p.usage {
                                col = col.push(labelled_info(
                                    fl!("usage"),
                                    bytes_to_pretty(&usage.used, false),
                                ));
                            }

                            if let Some(mount_point) = p.mount_points.first() {
                                col = col.push(link_info(
                                    fl!("mounted-at"),
                                    mount_point,
                                    Message::OpenPath(mount_point.clone()),
                                ));
                            }

                            col = col
                                .push(labelled_info(fl!("contents"), &type_str))
                                .push(labelled_info(
                                    fl!("device"),
                                    match p.device_path {
                                        Some(s) => s,
                                        None => fl!("unresolved"),
                                    },
                                ))
                                .push(labelled_info(fl!("uuid"), &p.uuid));

                            col
                        }
                        None => iced_widget::column![
                            heading(&segment.label),
                            labelled_info("Size", bytes_to_pretty(&segment.size, true)),
                        ]
                        .spacing(5),
                    }
                };

                let partition_type = match &drive.partition_table_type {
                    Some(t) => t.clone().to_uppercase(),
                    None => "Unknown".into(),
                };

                let can_remove = drive.is_loop || (drive.removable && drive.can_power_off);

                let mut drive_header = iced_widget::Row::new()
                    .push(heading(drive.name()))
                    .push(Space::new(Length::Fill, 0))
                    .spacing(10)
                    .width(Length::Fill);

                if can_remove {
                    drive_header = drive_header.push(
                        widget::button::custom(icon::from_name("media-eject-symbolic"))
                            .on_press(Message::Eject),
                    );
                }

                let drive_info = if drive.is_loop {
                    iced_widget::column![
                        drive_header,
                        Space::new(0, 10),
                        labelled_info("Size", bytes_to_pretty(&drive.size, true)),
                        labelled_info("Backing File", drive.backing_file.as_deref().unwrap_or(""),),
                    ]
                    .spacing(5)
                    .width(Length::Fill)
                } else {
                    iced_widget::column![
                        drive_header,
                        Space::new(0, 10),
                        labelled_info("Model", &drive.model),
                        labelled_info("Serial", &drive.serial),
                        labelled_info("Size", bytes_to_pretty(&drive.size, true)),
                        labelled_info("Partitioning", &partition_type),
                    ]
                    .spacing(5)
                    .width(Length::Fill)
                };
                iced_widget::column![
                    drive_info,
                    iced_widget::column![
                        heading("Volumes"),
                        Space::new(0, 10),
                        volumes_control.view()
                    ]
                    .spacing(5)
                    .width(Length::Fill),
                    info
                ]
                .spacing(60)
                .padding(20)
                .width(Length::Fill)
                .into()
            }
        }
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        struct DiskEventSubscription;

        Subscription::batch(vec![
            // Create a subscription which emits updates through a channel.
            Subscription::run_with_id(
                std::any::TypeId::of::<DiskEventSubscription>(),
                cosmic::iced::stream::channel(4, move |mut c| async move {
                    let manager = match DiskManager::new().await {
                        Ok(m) => m,
                        Err(e) => {
                            println!("Error creating DiskManager: {e}");
                            return;
                        }
                    };
                    let mut stream = match manager.device_event_stream_signals().await {
                        Ok(stream) => stream,
                        Err(e) => {
                            eprintln!(
                                "Device updates unavailable (failed to subscribe to UDisks2 signals): {e}"
                            );
                            return;
                        }
                    };

                    while let Some(event) = stream.next().await {
                        match event {
                            disks_dbus::DeviceEvent::Added(s) => {
                                let _ = c.send(Message::DriveAdded(s)).await;
                            }
                            disks_dbus::DeviceEvent::Removed(s) => {
                                let _ = c.send(Message::DriveRemoved(s)).await;
                            }
                        }
                    }
                }),
            ),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }
            Message::OpenPath(path) => {
                _ = open::that_detached(path);
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
            Message::VolumesMessage(message) => {
                let volumes_control = self.nav.active_data_mut::<VolumesControl>().unwrap(); //TODO: HANDLE UNWRAP.
                return volumes_control.update(message, &mut self.dialog);
            }

            Message::FormatDisk(msg) => {
                let Some(ShowDialog::FormatDisk(state)) = self.dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    FormatDiskMessage::EraseUpdate(v) => state.erase_index = v,
                    FormatDiskMessage::PartitioningUpdate(v) => state.partitioning_index = v,
                    FormatDiskMessage::Cancel => {
                        self.dialog = None;
                    }
                    FormatDiskMessage::Confirm => {
                        if state.running {
                            return Task::none();
                        }

                        state.running = true;

                        let drive = state.drive.clone();
                        let selected = drive.block_path.clone();
                        let erase = state.erase_index == 1;
                        let format_type = match state.partitioning_index {
                            0 => "dos",
                            1 => "gpt",
                            _ => "empty",
                        };

                        return Task::perform(
                            async move {
                                drive.format_disk(format_type, erase).await?;
                                DriveModel::get_drives().await
                            },
                            move |res| match res {
                                Ok(drives) => {
                                    Message::UpdateNav(drives, Some(selected.clone())).into()
                                }
                                Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                                    title: fl!("app-title"),
                                    body: format!("{e:#}"),
                                }))
                                .into(),
                            },
                        );
                    }
                };
            }
            Message::DriveRemoved(_drive_model) => {
                //TODO: use DeviceManager.apply_change()

                return Task::perform(
                    async {
                        match DriveModel::get_drives().await {
                            Ok(drives) => Some(drives),
                            Err(e) => {
                                println!("Error: {e}");
                                None
                            }
                        }
                    },
                    move |drives| match drives {
                        None => Message::None.into(),
                        Some(drives) => Message::UpdateNav(drives, None).into(),
                    },
                );
            }
            Message::DriveAdded(_drive_model) => {
                return Task::perform(
                    async {
                        match DriveModel::get_drives().await {
                            Ok(drives) => Some(drives),
                            Err(e) => {
                                println!("Error: {e}");
                                None
                            }
                        }
                    },
                    move |drives| match drives {
                        None => Message::None.into(),
                        Some(drives) => Message::UpdateNav(drives, None).into(),
                    },
                );
            }
            Message::None => {}
            Message::UpdateNav(drive_models, selected) => {
                // Some actions (unlock/format/create/delete) trigger a refresh; close the dialog if
                // it is in a running state so it doesn't linger after success.
                let should_close = match self.dialog.as_ref() {
                    Some(ShowDialog::UnlockEncrypted(s)) => s.running,
                    Some(ShowDialog::FormatDisk(s)) => s.running,
                    Some(ShowDialog::AddPartition(s)) => s.running,
                    Some(ShowDialog::FormatPartition(s)) => s.running,
                    Some(ShowDialog::EditPartition(s)) => s.running,
                    Some(ShowDialog::ResizePartition(s)) => s.running,
                    Some(ShowDialog::EditFilesystemLabel(s)) => s.running,
                    Some(ShowDialog::EditMountOptions(s)) => s.running,
                    Some(ShowDialog::ConfirmAction(s)) => s.running,
                    Some(ShowDialog::TakeOwnership(s)) => s.running,
                    Some(ShowDialog::ChangePassphrase(s)) => s.running,
                    Some(ShowDialog::EditEncryptionOptions(s)) => s.running,
                    Some(ShowDialog::DeletePartition(s)) => s.running,
                    _ => false,
                };

                if should_close {
                    self.dialog = None;
                }

                let selected = match selected {
                    Some(s) => Some(s),
                    None => self
                        .nav
                        .active_data::<DriveModel>()
                        .map(|d| d.block_path.clone()),
                };

                // Volumes-level preference; keep it stable across nav rebuilds.
                let show_reserved = self
                    .nav
                    .active_data::<VolumesControl>()
                    .map(|v| v.show_reserved)
                    .unwrap_or(false);

                self.nav.clear();

                let selected = match selected {
                    Some(s) => Some(s),
                    None => {
                        if selected.is_none() && !drive_models.is_empty() {
                            Some(drive_models.first().unwrap().block_path.clone())
                        } else {
                            None
                        }
                    }
                };

                for drive in drive_models {
                    let icon = match drive.removable {
                        true => "drive-removable-media-symbolic",
                        false => "disks-symbolic",
                    };

                    match selected {
                        Some(ref s) => {
                            if drive.block_path == s.clone() {
                                self.nav
                                    .insert()
                                    .text(drive.name())
                                    .data::<VolumesControl>(VolumesControl::new(
                                        drive.clone(),
                                        show_reserved,
                                    ))
                                    .data::<DriveModel>(drive)
                                    .icon(icon::from_name(icon))
                                    .activate();
                            } else {
                                self.nav
                                    .insert()
                                    .text(drive.name())
                                    .data::<VolumesControl>(VolumesControl::new(
                                        drive.clone(),
                                        show_reserved,
                                    ))
                                    .data::<DriveModel>(drive)
                                    .icon(icon::from_name(icon));
                            }
                        }
                        None => {
                            self.nav
                                .insert()
                                .text(drive.name())
                                .data::<VolumesControl>(VolumesControl::new(
                                    drive.clone(),
                                    show_reserved,
                                ))
                                .data::<DriveModel>(drive)
                                .icon(icon::from_name(icon));
                        }
                    }
                }
            }
            Message::Dialog(show_dialog) => self.dialog = Some(*show_dialog),
            Message::CloseDialog => {
                self.dialog = None;
            }
            Message::Eject => {
                let Some(drive) = self.nav.active_data::<DriveModel>().cloned() else {
                    return Task::none();
                };

                return Task::perform(
                    async move {
                        let res = drive.remove().await;
                        let drives = DriveModel::get_drives().await.ok();
                        (res, drives)
                    },
                    |(res, drives)| match res {
                        Ok(()) => match drives {
                            Some(drives) => Message::UpdateNav(drives, None).into(),
                            None => Message::None.into(),
                        },
                        Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("app-title"),
                            body: e.to_string(),
                        }))
                        .into(),
                    },
                );
            }
            Message::PowerOff => {
                let Some(drive) = self.nav.active_data::<DriveModel>().cloned() else {
                    return Task::none();
                };

                return Task::perform(
                    async move {
                        let res = drive.power_off().await;
                        let drives = DriveModel::get_drives().await.ok();
                        (res, drives)
                    },
                    |(res, drives)| match res {
                        Ok(()) => match drives {
                            Some(drives) => Message::UpdateNav(drives, None).into(),
                            None => Message::None.into(),
                        },
                        Err(e) => Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("app-title"),
                            body: e.to_string(),
                        }))
                        .into(),
                    },
                );
            }
            Message::Format => {
                let Some(drive) = self.nav.active_data::<DriveModel>().cloned() else {
                    return Task::none();
                };

                let partitioning_index = match drive.partition_table_type.as_deref() {
                    Some("dos") => 0,
                    Some("gpt") => 1,
                    _ => 2,
                };

                self.dialog = Some(ShowDialog::FormatDisk(FormatDiskDialog {
                    drive,
                    erase_index: 0,
                    partitioning_index,
                    running: false,
                }));
            }
            Message::SmartData => {
                let Some(drive) = self.nav.active_data::<DriveModel>().cloned() else {
                    return Task::none();
                };

                self.dialog = Some(ShowDialog::SmartData(SmartDataDialog {
                    drive: drive.clone(),
                    running: true,
                    info: None,
                    error: None,
                }));

                return Task::perform(
                    async move { drive.smart_info().await.map_err(|e| e.to_string()) },
                    |res| Message::SmartDialog(SmartDialogMessage::Loaded(res)).into(),
                );
            }
            Message::StandbyNow => {
                let Some(drive) = self.nav.active_data::<DriveModel>().cloned() else {
                    return Task::none();
                };

                return Task::perform(
                    async move { drive.standby_now().await.map_err(|e| e.to_string()) },
                    |res| {
                        Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("app-title"),
                            body: match res {
                                Ok(()) => "Standby requested.".to_string(),
                                Err(e) => e,
                            },
                        }))
                        .into()
                    },
                );
            }
            Message::Wakeup => {
                let Some(drive) = self.nav.active_data::<DriveModel>().cloned() else {
                    return Task::none();
                };

                return Task::perform(
                    async move { drive.wakeup().await.map_err(|e| e.to_string()) },
                    |res| {
                        Message::Dialog(Box::new(ShowDialog::Info {
                            title: fl!("app-title"),
                            body: match res {
                                Ok(()) => "Wake-up requested.".to_string(),
                                Err(e) => e,
                            },
                        }))
                        .into()
                    },
                );
            }
            Message::SmartDialog(msg) => {
                let Some(ShowDialog::SmartData(state)) = self.dialog.clone() else {
                    return Task::none();
                };

                match msg {
                    SmartDialogMessage::Close => {
                        self.dialog = None;
                    }
                    SmartDialogMessage::Loaded(res) => {
                        let mut next = state;
                        next.running = false;
                        match res {
                            Ok(info) => {
                                next.info = Some(info);
                                next.error = None;
                            }
                            Err(e) => {
                                eprintln!("SMART dialog error: {e}");
                                next.error = Some(e);
                            }
                        }
                        self.dialog = Some(ShowDialog::SmartData(next));
                    }
                    SmartDialogMessage::Refresh => {
                        let drive = state.drive.clone();
                        self.dialog = Some(ShowDialog::SmartData(SmartDataDialog {
                            drive: drive.clone(),
                            running: true,
                            info: state.info.clone(),
                            error: None,
                        }));

                        return Task::perform(
                            async move { drive.smart_info().await.map_err(|e| e.to_string()) },
                            |res| Message::SmartDialog(SmartDialogMessage::Loaded(res)).into(),
                        );
                    }
                    SmartDialogMessage::SelfTestShort => {
                        let drive = state.drive.clone();
                        self.dialog = Some(ShowDialog::SmartData(SmartDataDialog {
                            drive: drive.clone(),
                            running: true,
                            info: state.info.clone(),
                            error: None,
                        }));
                        return Task::perform(
                            async move {
                                drive
                                    .smart_selftest_start(disks_dbus::SmartSelfTestKind::Short)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            |res| {
                                Message::SmartDialog(SmartDialogMessage::ActionComplete(res)).into()
                            },
                        );
                    }
                    SmartDialogMessage::SelfTestExtended => {
                        let drive = state.drive.clone();
                        self.dialog = Some(ShowDialog::SmartData(SmartDataDialog {
                            drive: drive.clone(),
                            running: true,
                            info: state.info.clone(),
                            error: None,
                        }));
                        return Task::perform(
                            async move {
                                drive
                                    .smart_selftest_start(disks_dbus::SmartSelfTestKind::Extended)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            |res| {
                                Message::SmartDialog(SmartDialogMessage::ActionComplete(res)).into()
                            },
                        );
                    }
                    SmartDialogMessage::AbortSelfTest => {
                        let drive = state.drive.clone();
                        self.dialog = Some(ShowDialog::SmartData(SmartDataDialog {
                            drive: drive.clone(),
                            running: true,
                            info: state.info.clone(),
                            error: None,
                        }));
                        return Task::perform(
                            async move {
                                drive
                                    .smart_selftest_abort()
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            |res| {
                                Message::SmartDialog(SmartDialogMessage::ActionComplete(res)).into()
                            },
                        );
                    }
                    SmartDialogMessage::ActionComplete(res) => {
                        let drive = state.drive.clone();
                        let ok = res.is_ok();

                        let mut next = state;
                        next.running = false;
                        next.error = res.err();
                        if let Some(ref e) = next.error {
                            eprintln!("SMART dialog action error: {e}");
                        }
                        self.dialog = Some(ShowDialog::SmartData(next));

                        // After a successful action, refresh SMART data.
                        if ok {
                            return Task::perform(
                                async move { drive.smart_info().await.map_err(|e| e.to_string()) },
                                |res| Message::SmartDialog(SmartDialogMessage::Loaded(res)).into(),
                            );
                        }
                    }
                }
            }
            Message::NewDiskImage => {
                self.dialog = Some(ShowDialog::NewDiskImage(Box::new(NewDiskImageDialog {
                    path: String::new(),
                    size_bytes: 16 * 1024 * 1024,
                    running: false,
                    error: None,
                })));
            }
            Message::AttachDisk => {
                self.dialog = Some(ShowDialog::AttachDiskImage(Box::new(
                    AttachDiskImageDialog {
                        path: String::new(),
                        running: false,
                        error: None,
                    },
                )));
            }
            Message::CreateDiskFrom => {
                let Some(drive) = self.nav.active_data::<DriveModel>().cloned() else {
                    return Task::none();
                };

                self.dialog = Some(ShowDialog::ImageOperation(
                    ImageOperationDialog {
                        kind: ImageOperationKind::CreateFromDrive,
                        drive,
                        partition: None,
                        image_path: String::new(),
                        running: false,
                        error: None,
                    }
                    .into(),
                ));
            }
            Message::RestoreImageTo => {
                let Some(drive) = self.nav.active_data::<DriveModel>().cloned() else {
                    return Task::none();
                };

                self.dialog = Some(ShowDialog::ImageOperation(
                    ImageOperationDialog {
                        kind: ImageOperationKind::RestoreToDrive,
                        drive,
                        partition: None,
                        image_path: String::new(),
                        running: false,
                        error: None,
                    }
                    .into(),
                ));
            }
            Message::CreateDiskFromPartition => {
                let Some(drive) = self.nav.active_data::<DriveModel>().cloned() else {
                    return Task::none();
                };

                let Some(volumes_control) = self.nav.active_data::<VolumesControl>() else {
                    self.dialog = Some(ShowDialog::Info {
                        title: fl!("app-title"),
                        body: fl!("no-disk-selected"),
                    });
                    return Task::none();
                };

                let partition = volumes_control
                    .segments
                    .get(volumes_control.selected_segment)
                    .and_then(|s| s.volume.clone());

                let Some(partition) = partition else {
                    self.dialog = Some(ShowDialog::Info {
                        title: fl!("app-title"),
                        body: "Select a partition to create an image from.".to_string(),
                    });
                    return Task::none();
                };

                self.dialog = Some(ShowDialog::ImageOperation(
                    ImageOperationDialog {
                        kind: ImageOperationKind::CreateFromPartition,
                        drive,
                        partition: Some(partition),
                        image_path: String::new(),
                        running: false,
                        error: None,
                    }
                    .into(),
                ));
            }
            Message::RestoreImageToPartition => {
                let Some(drive) = self.nav.active_data::<DriveModel>().cloned() else {
                    return Task::none();
                };

                let Some(volumes_control) = self.nav.active_data::<VolumesControl>() else {
                    self.dialog = Some(ShowDialog::Info {
                        title: fl!("app-title"),
                        body: fl!("no-disk-selected"),
                    });
                    return Task::none();
                };

                let partition = volumes_control
                    .segments
                    .get(volumes_control.selected_segment)
                    .and_then(|s| s.volume.clone());

                let Some(partition) = partition else {
                    self.dialog = Some(ShowDialog::Info {
                        title: fl!("app-title"),
                        body: "Select a partition to restore an image to.".to_string(),
                    });
                    return Task::none();
                };

                self.dialog = Some(ShowDialog::ImageOperation(
                    ImageOperationDialog {
                        kind: ImageOperationKind::RestoreToPartition,
                        drive,
                        partition: Some(partition),
                        image_path: String::new(),
                        running: false,
                        error: None,
                    }
                    .into(),
                ));
            }
            Message::NewDiskImageDialog(msg) => {
                let Some(ShowDialog::NewDiskImage(state)) = self.dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    NewDiskImageDialogMessage::PathUpdate(v) => state.path = v,
                    NewDiskImageDialogMessage::SizeUpdate(v) => state.size_bytes = v,
                    NewDiskImageDialogMessage::Cancel => {
                        self.dialog = None;
                    }
                    NewDiskImageDialogMessage::Create => {
                        if state.running {
                            return Task::none();
                        }

                        let path = state.path.clone();
                        let size_bytes = state.size_bytes;

                        state.running = true;
                        state.error = None;

                        return Task::perform(
                            async move {
                                if path.trim().is_empty() {
                                    anyhow::bail!("Destination path is required");
                                }

                                let file = OpenOptions::new()
                                    .write(true)
                                    .create_new(true)
                                    .open(&path)
                                    .await?;
                                file.set_len(size_bytes).await?;
                                Ok(())
                            },
                            |res: anyhow::Result<()>| {
                                Message::NewDiskImageDialog(NewDiskImageDialogMessage::Complete(
                                    res.map_err(|e| e.to_string()),
                                ))
                                .into()
                            },
                        );
                    }
                    NewDiskImageDialogMessage::Complete(res) => {
                        state.running = false;
                        match res {
                            Ok(()) => {
                                self.dialog = Some(ShowDialog::Info {
                                    title: fl!("app-title"),
                                    body: "Disk image created.".to_string(),
                                });
                            }
                            Err(e) => {
                                eprintln!("New disk image dialog error: {e}");
                                state.error = Some(e);
                            }
                        }
                    }
                }
            }
            Message::AttachDiskImageDialog(msg) => {
                let Some(ShowDialog::AttachDiskImage(state)) = self.dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    AttachDiskImageDialogMessage::PathUpdate(v) => state.path = v,
                    AttachDiskImageDialogMessage::Cancel => {
                        if !state.running {
                            self.dialog = None;
                        }
                    }
                    AttachDiskImageDialogMessage::Attach => {
                        if state.running {
                            return Task::none();
                        }

                        let path = state.path.clone();
                        state.running = true;
                        state.error = None;

                        return Task::perform(
                            async move {
                                if path.trim().is_empty() {
                                    anyhow::bail!("Image file path is required");
                                }

                                let block_object_path = disks_dbus::loop_setup(&path).await?;

                                match disks_dbus::mount_filesystem(block_object_path.clone()).await
                                {
                                    Ok(()) => Ok(AttachDiskResult {
                                        mounted: true,
                                        message: "Attached and mounted image.".to_string(),
                                    }),
                                    Err(e) => {
                                        eprintln!("Attach image: mount attempt failed: {e}");
                                        Ok(AttachDiskResult {
                                        mounted: false,
                                        message: "Attached image. If it contains partitions, select and mount them from the main view.".to_string(),
                                        })
                                    }
                                }
                            },
                            |res: anyhow::Result<AttachDiskResult>| {
                                Message::AttachDiskImageDialog(
                                    AttachDiskImageDialogMessage::Complete(
                                        res.map_err(|e| e.to_string()),
                                    ),
                                )
                                .into()
                            },
                        );
                    }
                    AttachDiskImageDialogMessage::Complete(res) => {
                        state.running = false;
                        match res {
                            Ok(r) => {
                                self.dialog = Some(ShowDialog::Info {
                                    title: fl!("app-title"),
                                    body: r.message,
                                });

                                return Task::perform(
                                    async { DriveModel::get_drives().await.ok() },
                                    |drives| match drives {
                                        None => Message::None.into(),
                                        Some(drives) => Message::UpdateNav(drives, None).into(),
                                    },
                                );
                            }
                            Err(e) => {
                                eprintln!("Attach disk image dialog error: {e}");
                                state.error = Some(e);
                            }
                        }
                    }
                }
            }
            Message::ImageOperationDialog(msg) => {
                let Some(ShowDialog::ImageOperation(state)) = self.dialog.as_mut() else {
                    return Task::none();
                };

                match msg {
                    ImageOperationDialogMessage::PathUpdate(v) => state.image_path = v,
                    ImageOperationDialogMessage::CancelOperation => {
                        if state.running {
                            if let Some(flag) = self.image_op_cancel.as_ref() {
                                flag.store(true, Ordering::SeqCst);
                            }
                        } else {
                            self.dialog = None;
                        }
                    }
                    ImageOperationDialogMessage::Start => {
                        if state.running {
                            return Task::none();
                        }

                        let image_path = state.image_path.clone();
                        if image_path.trim().is_empty() {
                            let e = "Image path is required".to_string();
                            eprintln!("Image operation dialog error: {e}");
                            state.error = Some(e);
                            return Task::none();
                        }

                        let kind = state.kind;
                        let drive = state.drive.clone();
                        let partition = state.partition.clone();

                        let cancel = Arc::new(AtomicBool::new(false));
                        self.image_op_cancel = Some(cancel.clone());

                        state.running = true;
                        state.error = None;

                        return Task::perform(
                            async move {
                                run_image_operation(kind, drive, partition, image_path, cancel)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            |res| {
                                Message::ImageOperationDialog(
                                    ImageOperationDialogMessage::Complete(res),
                                )
                                .into()
                            },
                        );
                    }
                    ImageOperationDialogMessage::Complete(res) => {
                        self.image_op_cancel = None;
                        state.running = false;
                        match res {
                            Ok(()) => {
                                self.dialog = Some(ShowDialog::Info {
                                    title: fl!("app-title"),
                                    body: "Operation completed.".to_string(),
                                });

                                return Task::perform(
                                    async { DriveModel::get_drives().await.ok() },
                                    |drives| match drives {
                                        None => Message::None.into(),
                                        Some(drives) => Message::UpdateNav(drives, None).into(),
                                    },
                                );
                            }
                            Err(e) => {
                                eprintln!("Image operation dialog error: {e}");
                                state.error = Some(e);
                            }
                        }
                    }
                }
            }
            Message::Surface(action) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(action),
                ));
            }
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        // Activate the page in the model.
        if self.dialog.is_none() {
            let previous_show_reserved = self
                .nav
                .active_data::<VolumesControl>()
                .map(|v| v.show_reserved);

            self.nav.activate(id);

            if let Some(show_reserved) = previous_show_reserved
                && let Some(volumes_control) = self.nav.active_data_mut::<VolumesControl>()
            {
                volumes_control.set_show_reserved(show_reserved);
            }

            self.update_title()
        } else {
            Task::none()
        }
    }
}

impl AppModel {
    /// The about page for this app.
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<Message> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}

async fn copy_with_cancel<R, W>(
    mut reader: R,
    mut writer: W,
    cancel: Arc<AtomicBool>,
) -> anyhow::Result<u64>
where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut buf = vec![0u8; 4 * 1024 * 1024];
    let mut total: u64 = 0;

    loop {
        if cancel.load(Ordering::Relaxed) {
            anyhow::bail!("Cancelled");
        }

        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        writer.write_all(&buf[..n]).await?;
        total = total.saturating_add(n as u64);
    }

    writer.flush().await?;
    Ok(total)
}

async fn run_image_operation(
    kind: ImageOperationKind,
    drive: DriveModel,
    partition: Option<disks_dbus::VolumeModel>,
    image_path: String,
    cancel: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    match kind {
        ImageOperationKind::CreateFromDrive => {
            let fd = drive.open_for_backup().await?;
            let reader = tokio::fs::File::from_std(std::fs::File::from(fd));
            let writer = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&image_path)
                .await?;

            let _bytes = copy_with_cancel(reader, writer, cancel).await?;
            Ok(())
        }
        ImageOperationKind::CreateFromPartition => {
            let Some(partition) = partition else {
                anyhow::bail!("No partition selected");
            };

            let fd = partition.open_for_backup().await?;
            let reader = tokio::fs::File::from_std(std::fs::File::from(fd));
            let writer = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&image_path)
                .await?;

            let _bytes = copy_with_cancel(reader, writer, cancel).await?;
            Ok(())
        }
        ImageOperationKind::RestoreToDrive => {
            // Preflight: attempt to unmount all mounted partitions.
            for p in &drive.volumes_flat {
                if p.is_mounted() {
                    p.unmount().await?;
                }
            }

            let src_meta = tokio::fs::metadata(&image_path).await?;
            if src_meta.len() > drive.size {
                anyhow::bail!(
                    "Image is larger than the selected drive (image={} bytes, drive={} bytes)",
                    src_meta.len(),
                    drive.size
                );
            }

            let src = tokio::fs::File::open(&image_path).await?;
            let fd = drive.open_for_restore().await?;
            let dest = tokio::fs::File::from_std(std::fs::File::from(fd));

            let _bytes = copy_with_cancel(src, dest, cancel).await?;
            Ok(())
        }
        ImageOperationKind::RestoreToPartition => {
            let Some(partition) = partition else {
                anyhow::bail!("No partition selected");
            };

            if partition.is_mounted() {
                partition.unmount().await?;
            }

            let src_meta = tokio::fs::metadata(&image_path).await?;
            if src_meta.len() > partition.size {
                anyhow::bail!(
                    "Image is larger than the selected partition (image={} bytes, partition={} bytes)",
                    src_meta.len(),
                    partition.size
                );
            }

            let src = tokio::fs::File::open(&image_path).await?;
            let fd = partition.open_for_restore().await?;
            let dest = tokio::fs::File::from_std(std::fs::File::from(fd));

            let _bytes = copy_with_cancel(src, dest, cancel).await?;
            Ok(())
        }
    }
}
