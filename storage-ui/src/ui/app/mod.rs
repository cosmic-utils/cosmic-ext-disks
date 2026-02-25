pub(crate) mod message;
pub(crate) mod state;
pub(crate) mod subscriptions;
pub(crate) mod update;
pub(crate) mod view;

pub(crate) use message::Message;
pub(crate) use state::{AppModel, ContextPage};

use crate::client::FilesystemsClient;
use crate::client::RcloneClient;
use crate::models::load_all_drives;

use crate::config::Config;
use crate::ui::network::NetworkState;
use crate::ui::sidebar::SidebarState;
use cosmic::app::{Core, Task};
use cosmic::widget::nav_bar;
use cosmic::{Application, Element};

pub(crate) const APP_ID: &str = "com.cosmic.ext.Storage";

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
            sidebar: SidebarState::default(),
            dialog: None,
            image_op_operation_id: None,
            filesystem_tools: vec![],
            network: NetworkState::new(),
            // Optional configuration file for an application.
            config: Config::load(Self::APP_ID),
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        let nav_command = Task::perform(
            async {
                match load_all_drives().await {
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

        // Load filesystem tools from service
        let tools_command = Task::perform(
            async {
                match FilesystemsClient::new().await {
                    Ok(client) => match client.get_filesystem_tools().await {
                        Ok(tools) => Some(tools),
                        Err(e) => {
                            tracing::error!(%e, "failed to load filesystem tools");
                            None
                        }
                    },
                    Err(e) => {
                        tracing::error!(%e, "failed to create filesystems client");
                        None
                    }
                }
            },
            |tools| match tools {
                None => Message::None.into(),
                Some(tools) => Message::FilesystemToolsLoaded(tools).into(),
            },
        );

        // Load network remotes from RClone service
        let network_command = Task::perform(
            async {
                match RcloneClient::new().await {
                    Ok(client) => match client.list_remotes().await {
                        Ok(list) => Some(list.remotes),
                        Err(e) => {
                            tracing::warn!(%e, "failed to load network remotes");
                            None
                        }
                    },
                    Err(e) => {
                        tracing::info!(%e, "RClone client not available, network features disabled");
                        None
                    }
                }
            },
            |remotes| {
                Message::NetworkRemotesLoaded(
                    remotes.ok_or_else(|| "RClone not available".to_string()),
                )
                .into()
            },
        );

        (
            app,
            command
                .chain(nav_command)
                .chain(tools_command)
                .chain(network_command),
        )
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        view::header_start(self)
    }

    /// Elements to pack at the center of the header bar.
    fn header_center(&self) -> Vec<Element<'_, Self::Message>> {
        view::header_center(self)
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
