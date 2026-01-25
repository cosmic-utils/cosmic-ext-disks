use std::collections::HashMap;
use std::sync::LazyLock;

use crate::app::{ContextPage, Message};
use crate::fl;
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
                fl!("menu-image"),
                vec![
                    menu::Item::Button(fl!("new-disk-image"), None, MenuAction::NewDiskImage),
                    menu::Item::Button(fl!("attach-disk-image"), None, MenuAction::AttachDisk),
                    menu::Item::Button(
                        fl!("create-disk-from-drive"),
                        None,
                        MenuAction::CreateDiskFrom,
                    ),
                    menu::Item::Button(
                        fl!("restore-image-to-drive"),
                        None,
                        MenuAction::RestoreImageTo,
                    ),
                    menu::Item::Button(
                        fl!("create-disk-from-partition"),
                        None,
                        MenuAction::CreateDiskFromPartition,
                    ),
                    menu::Item::Button(
                        fl!("restore-image-to-partition"),
                        None,
                        MenuAction::RestoreImageToPartition,
                    ),
                ],
            ),
            (
                fl!("menu-disk"),
                vec![
                    menu::Item::Button(fl!("eject"), None, MenuAction::Eject),
                    menu::Item::Button(fl!("power-off"), None, MenuAction::PowerOff),
                    menu::Item::Button(fl!("format-disk"), None, MenuAction::Format),
                    menu::Item::Button(fl!("smart-data-self-tests"), None, MenuAction::SmartData),
                    menu::Item::Button(fl!("standby-now"), None, MenuAction::StandbyNow),
                    menu::Item::Button(fl!("wake-up-from-standby"), None, MenuAction::Wakeup),
                ],
            ),
            (
                fl!("menu-view"),
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
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
    SmartData,
    StandbyNow,
    Wakeup,
    NewDiskImage,
    AttachDisk,
    CreateDiskFrom,
    RestoreImageTo,
    CreateDiskFromPartition,
    RestoreImageToPartition,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Eject => Message::Eject,
            MenuAction::PowerOff => Message::PowerOff,
            MenuAction::Format => Message::Format,
            MenuAction::SmartData => Message::SmartData,
            MenuAction::StandbyNow => Message::StandbyNow,
            MenuAction::Wakeup => Message::Wakeup,
            MenuAction::NewDiskImage => Message::NewDiskImage,
            MenuAction::AttachDisk => Message::AttachDisk,
            MenuAction::CreateDiskFrom => Message::CreateDiskFrom,
            MenuAction::RestoreImageTo => Message::RestoreImageTo,
            MenuAction::CreateDiskFromPartition => Message::CreateDiskFromPartition,
            MenuAction::RestoreImageToPartition => Message::RestoreImageToPartition,
        }
    }
}
