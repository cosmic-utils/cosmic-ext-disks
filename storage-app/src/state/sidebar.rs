use crate::models::UiDrive;
use cosmic::widget::nav_bar;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

fn compare_drive_sort_keys(
    left_device: &str,
    left_name: &str,
    right_device: &str,
    right_name: &str,
) -> Ordering {
    left_device
        .cmp(right_device)
        .then_with(|| left_name.cmp(right_name))
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SidebarNodeKey {
    Drive(String),
    Volume(String),
    Logical(String),
}

#[derive(Debug, Default)]
pub struct SidebarState {
    /// Latest drive models used to render the tree.
    pub drives: Vec<UiDrive>,

    /// Mapping from drive `block_path` to the corresponding `nav_bar::Id` in `app.nav`.
    pub drive_entities: HashMap<String, nav_bar::Id>,

    /// Expanded nodes in the tree.
    pub expanded: HashSet<SidebarNodeKey>,

    /// Selected (focused) child node. Drive selection is still managed via `app.nav`.
    pub selected_child: Option<SidebarNodeKey>,

    /// Logical section loading status.
    pub logical_loading: bool,

    /// Physical/image drive sections loading status.
    pub drives_loading: bool,

    /// Network section loading status.
    pub network_loading: bool,

    /// Number of in-flight drive build tasks.
    pub drive_builds_pending: usize,

    /// Frame index for lightweight spinner animation.
    pub spinner_frame: u8,
}

impl SidebarState {
    pub fn active_drive_block_path(&self, app_nav: &nav_bar::Model) -> Option<String> {
        app_nav
            .active_data::<UiDrive>()
            .map(|d| d.device().to_string())
    }

    pub fn set_drives(&mut self, drives: Vec<UiDrive>) {
        self.drives = drives;
    }

    pub fn set_drive_entities(&mut self, entities: HashMap<String, nav_bar::Id>) {
        self.drive_entities = entities;
    }

    pub fn set_logical_loading(&mut self, loading: bool) {
        self.logical_loading = loading;
    }

    pub fn set_network_loading(&mut self, loading: bool) {
        self.network_loading = loading;
    }

    pub fn start_drive_loading(&mut self, total: usize) {
        self.drives_loading = true;
        self.drive_builds_pending = total;
    }

    pub fn finish_drive_loading(&mut self) {
        self.drives_loading = false;
        self.drive_builds_pending = 0;
    }

    pub fn mark_drive_build_finished(&mut self) -> bool {
        if self.drive_builds_pending > 0 {
            self.drive_builds_pending -= 1;
        }
        self.drive_builds_pending == 0
    }

    pub fn upsert_drive_sorted(&mut self, drive: UiDrive) {
        let device = drive.device().to_string();
        if let Some(existing_index) = self.drives.iter().position(|d| d.device() == device) {
            self.drives[existing_index] = drive;
        } else {
            self.drives.push(drive);
        }

        self.drives.sort_by(|left, right| {
            compare_drive_sort_keys(left.device(), &left.name(), right.device(), &right.name())
        });
    }

    pub fn has_active_loading(&self) -> bool {
        self.logical_loading || self.drives_loading || self.network_loading
    }

    pub fn advance_spinner_frame(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % 10;
    }

    pub fn is_expanded(&self, key: &SidebarNodeKey) -> bool {
        self.expanded.contains(key)
    }

    pub fn toggle_expanded(&mut self, key: SidebarNodeKey) {
        if !self.expanded.insert(key.clone()) {
            self.expanded.remove(&key);
        }
    }

    pub fn find_drive(&self, device: &str) -> Option<&UiDrive> {
        self.drives.iter().find(|d| d.device() == device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_flags_transition_correctly() {
        let mut state = SidebarState::default();

        assert!(!state.logical_loading);
        assert!(!state.network_loading);
        assert!(!state.drives_loading);
        assert_eq!(state.drive_builds_pending, 0);

        state.set_logical_loading(true);
        state.set_network_loading(true);
        state.start_drive_loading(2);

        assert!(state.logical_loading);
        assert!(state.network_loading);
        assert!(state.drives_loading);
        assert_eq!(state.drive_builds_pending, 2);

        assert!(!state.mark_drive_build_finished());
        assert_eq!(state.drive_builds_pending, 1);

        assert!(state.mark_drive_build_finished());
        assert_eq!(state.drive_builds_pending, 0);

        state.finish_drive_loading();
        state.set_logical_loading(false);
        state.set_network_loading(false);

        assert!(!state.has_active_loading());
    }

    #[test]
    fn spinner_frame_advances_and_wraps() {
        let mut state = SidebarState {
            spinner_frame: 9,
            ..Default::default()
        };

        state.advance_spinner_frame();
        assert_eq!(state.spinner_frame, 0);
    }

    #[test]
    fn drive_sort_keys_are_deterministic() {
        let mut keys = vec![
            ("/dev/loop7", "loop7"),
            ("/dev/nvme0n1", "nvme"),
            ("/dev/loop2", "loop2"),
        ];

        keys.sort_by(|(left_device, left_name), (right_device, right_name)| {
            compare_drive_sort_keys(left_device, left_name, right_device, right_name)
        });

        assert_eq!(
            keys,
            vec![
                ("/dev/loop2", "loop2"),
                ("/dev/loop7", "loop7"),
                ("/dev/nvme0n1", "nvme"),
            ]
        );
    }
}
