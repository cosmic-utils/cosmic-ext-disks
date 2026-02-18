// SPDX-License-Identifier: GPL-3.0-only

//! Network mount management UI components
//!
//! Provides components for managing RClone and future network mounts
//! (Samba, FTP) in the sidebar.

mod message;
mod state;
pub(crate) mod view;

pub(crate) use message::NetworkMessage;
pub(crate) use state::NetworkState;
