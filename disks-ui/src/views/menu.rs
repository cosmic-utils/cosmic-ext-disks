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
                fl!("menu-view"),
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        ],
    )]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}
