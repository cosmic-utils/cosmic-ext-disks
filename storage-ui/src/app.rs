// SPDX-License-Identifier: GPL-3.0-only

pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
pub const APP_ICON: &[u8] =
    include_bytes!("../resources/icons/hicolor/scalable/apps/com.cosmic.ext.Storage.svg");

pub use crate::ui::app::state::AppModel;

pub use crate::ui::app::message::Message;
