pub(crate) mod message;
pub(crate) mod state;
pub(crate) mod subscriptions;
pub(crate) mod update;
pub(crate) mod view;

pub(crate) use message::Message;
pub(crate) use state::{AppModel, ContextPage};

use crate::config::Config;
use cosmic::app::{Core, Task};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::widget::{menu, nav_bar};
use cosmic::{Application, Element};
use disks_dbus::DriveModel;
use std::collections::HashMap;

pub(crate) const APP_ID: &str = "com.cosmos.Disks";

/// Create a COSMIC application from the app model.
impl Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = APP_ID;

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
            key_binds: HashMap::<menu::KeyBind, crate::views::menu::MenuAction>::new(),
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
                        tracing::error!(%e, "failed to load drives");
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
        view::header_start(self)
    }

    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        view::dialog(self)
    }

    /// Allows overriding the default nav bar widget.
    fn nav_bar(&self) -> Option<Element<'_, cosmic::Action<Self::Message>>> {
        view::nav_bar(self)
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        view::nav_model(self)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(
        &self,
    ) -> Option<cosmic::app::context_drawer::ContextDrawer<'_, Self::Message>> {
        view::context_drawer(self)
    }

    /// Describes the interface based on the current state of the application model.
    fn view(&self) -> Element<'_, Self::Message> {
        view::view(self)
    }

    /// Register subscriptions for this application.
    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        subscriptions::subscription(self)
    }

    /// Handles messages emitted by the application and its widgets.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        update::update(self, message)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        update::on_nav_select(self, id)
    }
}
