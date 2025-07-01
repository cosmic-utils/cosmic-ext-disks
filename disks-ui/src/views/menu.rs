use std::collections::HashMap;
use std::sync::LazyLock;

use crate::app::{ContextPage, Message};
use cosmic::Core;
use cosmic::widget::Id;
use cosmic::{Element, widget::menu};

static MENU_ID: LazyLock<Id> = LazyLock::new(|| Id::new("menu_id"));

pub fn menu_view(
    core: &Core,
    key_binds: &HashMap<menu::KeyBind, MenuAction>,
) -> Vec<Element<'static, Message>> {
    vec![cosmic::widget::responsive_menu_bar().into_element(
        core, // Replace with `self.core()` if applicable
        key_binds,
        MENU_ID.clone(),
        Message::Surface,
        vec![
            (
                "Image".into(),
                vec![
                    menu::Item::Button("New Disk Image", None, MenuAction::NewDiskImage),
                    menu::Item::Button("Attach Disk Image", None, MenuAction::AttachDisk),
                    menu::Item::Button("Create Disk From Drive", None, MenuAction::CreateDiskFrom),
                    menu::Item::Button("Restore Image to Drive", None, MenuAction::RestoreImageTo),
                ],
            ),
            (
                "Disk".into(),
                vec![
                    menu::Item::Button("Eject", None, MenuAction::Eject),
                    menu::Item::Button("Power Off", None, MenuAction::PowerOff),
                    menu::Item::Button("Format Disk", None, MenuAction::Format),
                    menu::Item::Button("Benchmark Disk", None, MenuAction::Benchmark),
                    menu::Item::Button("SMART Data & Self-Tests", None, MenuAction::SmartData),
                    menu::Item::Button("Drive Settings", None, MenuAction::DriveSettings),
                    menu::Item::Button("Standby Now", None, MenuAction::StandbyNow),
                    menu::Item::Button("Wake-up From Standby", None, MenuAction::Wakeup),
                ],
            ),
            (
                "View".into(),
                vec![menu::Item::Button("about", None, MenuAction::About)],
            ),
        ],
    )]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
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
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Eject => Message::Eject,
            MenuAction::PowerOff => Message::PowerOff,
            MenuAction::Format => Message::Format,
            MenuAction::Benchmark => Message::Benchmark,
            MenuAction::SmartData => Message::SmartData,
            MenuAction::DriveSettings => Message::DriveSettings,
            MenuAction::StandbyNow => Message::StandbyNow,
            MenuAction::Wakeup => Message::Wakeup,
            MenuAction::NewDiskImage => Message::NewDiskImage,
            MenuAction::AttachDisk => Message::AttachDisk,
            MenuAction::CreateDiskFrom => Message::CreateDiskFrom,
            MenuAction::RestoreImageTo => Message::RestoreImageTo,
        }
    }
}
