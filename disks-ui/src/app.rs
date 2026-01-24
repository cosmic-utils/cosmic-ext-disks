// SPDX-License-Identifier: MPL-2.0

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
use disks_dbus::CreatePartitionInfo;
use disks_dbus::bytes_to_pretty;
use disks_dbus::{DiskManager, DriveModel};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::time::Duration;

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

    pub dialog: Option<ShowDialog>,
}

#[derive(Debug, Clone)]
pub enum ShowDialog {
    DeletePartition(String),
    AddPartition(CreatePartitionInfo),
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    OpenRepositoryUrl,
    OpenPath(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    LaunchUrl(String),
    VolumesMessage(VolumesControlMessage),
    DriveRemoved(String),
    DriveAdded(String),
    None,
    UpdateNav(Vec<DriveModel>, Option<String>),
    Dialog(ShowDialog),
    CloseDialog,
    Eject,
    PowerOff,
    Format,
    Benchmark,
    SmartData,
    DriveSettings,
    StandbyNow,
    Wakeup,
    NewDiskImage,
    AttachDisk,
    CreateDiskFrom,
    RestoreImageTo,
    Surface(cosmic::surface::Action),
}

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
                ShowDialog::DeletePartition(name) => Some(dialogs::confirmation(
                    fl!("delete", name = name),
                    fl!("delete-confirmation", name = name),
                    VolumesControlMessage::Delete.into(),
                    Some(Message::CloseDialog),
                )),

                ShowDialog::AddPartition(create) => Some(dialogs::create_partition(create.clone())),
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
                let info = match segment.partition.clone() {
                    Some(p) => {
                        let mut name = p.name.clone();
                        if name.is_empty() {
                            name = fl!("partition-number", number = p.number);
                        } else {
                            name =
                                fl!("partition-number-with-name", number = p.number, name = name);
                        }

                        let mut type_str = p.id_type.clone().to_uppercase();
                        type_str = format!("{} - {}", type_str, p.partition_type.clone());

                        match &p.usage {
                            Some(usage) => iced_widget::column![
                                heading(name),
                                Space::new(0, 10),
                                labelled_info(fl!("size"), bytes_to_pretty(&p.size, true)),
                                labelled_info(fl!("usage"), bytes_to_pretty(&usage.used, false)),
                                link_info(
                                    fl!("mounted-at"),
                                    &usage.mount_point,
                                    Message::OpenPath(usage.mount_point.clone())
                                ),
                                labelled_info(fl!("contents"), &type_str),
                                labelled_info(
                                    fl!("device"),
                                    match p.device_path {
                                        Some(s) => {
                                            s
                                        }
                                        None => fl!("unresolved"),
                                    }
                                ),
                                labelled_info(fl!("uuid"), &p.uuid),
                            ]
                            .spacing(5),

                            None => iced_widget::column![
                                heading(name),
                                Space::new(0, 10),
                                labelled_info(fl!("size"), bytes_to_pretty(&p.size, true)),
                                labelled_info(fl!("contents"), &type_str),
                                labelled_info(
                                    fl!("device"),
                                    match p.device_path {
                                        Some(s) => {
                                            s
                                        }
                                        None => fl!("unresolved"),
                                    }
                                ),
                                labelled_info(fl!("uuid"), &p.uuid),
                            ]
                            .spacing(5),
                        }
                    }
                    None => iced_widget::column![
                        heading(&segment.label),
                        labelled_info("Size", bytes_to_pretty(&segment.size, true)),
                    ]
                    .spacing(5),
                };

                let partition_type = match &drive.partition_table_type {
                    Some(t) => t.clone().to_uppercase(),
                    None => "Unknown".into(),
                };
                iced_widget::column![
                    iced_widget::column![
                        heading(drive.name()),
                        Space::new(0, 10),
                        labelled_info("Model", &drive.model),
                        labelled_info("Serial", &drive.serial),
                        labelled_info("Size", bytes_to_pretty(&drive.size, true)),
                        labelled_info("Partitioning", &partition_type),
                    ]
                    .spacing(5)
                    .width(Length::Fill),
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
                                "Falling back to polling-based device updates (signals unavailable): {e}"
                            );
                            manager.device_event_stream(Duration::from_secs(10))
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
                let selected = match selected {
                    Some(s) => Some(s),
                    None => self
                        .nav
                        .active_data::<DriveModel>()
                        .map(|d| d.block_path.clone()),
                };

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
                                    .data::<VolumesControl>(VolumesControl::new(drive.clone()))
                                    .data::<DriveModel>(drive)
                                    .icon(icon::from_name(icon))
                                    .activate();
                            } else {
                                self.nav
                                    .insert()
                                    .text(drive.name())
                                    .data::<VolumesControl>(VolumesControl::new(drive.clone()))
                                    .data::<DriveModel>(drive)
                                    .icon(icon::from_name(icon));
                            }
                        }
                        None => {
                            self.nav
                                .insert()
                                .text(drive.name())
                                .data::<VolumesControl>(VolumesControl::new(drive.clone()))
                                .data::<DriveModel>(drive)
                                .icon(icon::from_name(icon));
                        }
                    }
                }
            }
            Message::Dialog(show_dialog) => self.dialog = Some(show_dialog),
            Message::CloseDialog => {
                self.dialog = None;
            }
            Message::Eject => {
                if let Some(drive) = self.nav.active_data::<DriveModel>().cloned() {
                    return Task::perform(
                        async move {
                            let _ = drive.eject().await; //TODO handle error
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
            }
            Message::PowerOff => todo!(),
            Message::Format => todo!(),
            Message::Benchmark => todo!(),
            Message::SmartData => todo!(),
            Message::DriveSettings => todo!(),
            Message::StandbyNow => todo!(),
            Message::Wakeup => todo!(),
            Message::NewDiskImage => todo!(),
            Message::AttachDisk => todo!(),
            Message::CreateDiskFrom => todo!(),
            Message::RestoreImageTo => todo!(),
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
            self.nav.activate(id);
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

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}
