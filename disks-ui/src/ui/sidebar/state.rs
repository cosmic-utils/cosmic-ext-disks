use cosmic::widget::nav_bar;
use disks_dbus::DriveModel;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SidebarNodeKey {
    Drive(String),
    Volume(String),
}

#[derive(Debug, Default)]
pub struct SidebarState {
    /// Latest drive models used to render the tree.
    pub drives: Vec<DriveModel>,

    /// Mapping from drive `block_path` to the corresponding `nav_bar::Id` in `app.nav`.
    pub drive_entities: HashMap<String, nav_bar::Id>,

    /// Expanded nodes in the tree.
    pub expanded: HashSet<SidebarNodeKey>,

    /// Selected (focused) child node. Drive selection is still managed via `app.nav`.
    pub selected_child: Option<SidebarNodeKey>,
}

impl SidebarState {
    pub fn active_drive_block_path(&self, app_nav: &nav_bar::Model) -> Option<String> {
        app_nav
            .active_data::<DriveModel>()
            .map(|d| d.block_path.clone())
    }

    pub fn set_drives(&mut self, drives: Vec<DriveModel>) {
        self.drives = drives;
    }

    pub fn set_drive_entities(&mut self, entities: HashMap<String, nav_bar::Id>) {
        self.drive_entities = entities;
    }

    pub fn is_expanded(&self, key: &SidebarNodeKey) -> bool {
        self.expanded.contains(key)
    }

    pub fn toggle_expanded(&mut self, key: SidebarNodeKey) {
        if !self.expanded.insert(key.clone()) {
            self.expanded.remove(&key);
        }
    }

    pub fn find_drive(&self, block_path: &str) -> Option<DriveModel> {
        self.drives
            .iter()
            .find(|d| d.block_path == block_path)
            .cloned()
    }
}
