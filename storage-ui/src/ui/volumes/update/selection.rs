use cosmic::Task;

use crate::app::Message;
use crate::ui::btrfs::BtrfsState;
use crate::ui::dialogs::state::ShowDialog;
use crate::ui::volumes::helpers;

use crate::ui::volumes::VolumesControl;
use crate::ui::volumes::state::DetailTab;

pub(super) fn segment_selected(
    control: &mut VolumesControl,
    index: usize,
    dialog: &Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    if dialog.is_none() {
        let Some(last_index) = control.segments.len().checked_sub(1) else {
            control.selected_segment = 0;
            control.selected_volume = None;
            return Task::batch(vec![Task::done(cosmic::Action::App(
                Message::SidebarClearChildSelection,
            ))]);
        };

        let index = index.min(last_index);
        control.selected_segment = index;
        control.selected_volume = None;
        control.detail_tab = DetailTab::VolumeInfo;
        control.usage_state = super::super::state::UsageTabState::default();
        control.segments.iter_mut().for_each(|s| s.state = false);
        if let Some(segment) = control.segments.get_mut(index) {
            segment.state = true;
        }

        // Initialize/update BTRFS state if this segment has a BTRFS volume
        // Also checks through LUKS containers for an inner BTRFS filesystem
        if let Some(segment) = control.segments.get(index)
            && let Some(volume) = &segment.volume
        {
            let btrfs_info = helpers::detect_btrfs_for_volume(&control.volumes, volume);
            tracing::info!(
                "segment_selected: index={}, btrfs_detected={}, id_type={}, has_filesystem={}, mount_points={:?}",
                index,
                btrfs_info.is_some(),
                volume.id_type,
                volume.has_filesystem,
                volume.mount_points
            );
            if let Some((mount_point, block_path)) = btrfs_info {
                tracing::info!(
                    "segment_selected: Initializing BTRFS state with mount_point={:?}, block_path={}",
                    mount_point,
                    block_path
                );
                control.btrfs_state = Some(BtrfsState::new(
                    mount_point.clone(),
                    Some(block_path.clone()),
                ));

                // If mounted, trigger data loading
                if let Some(mp) = mount_point {
                    let device_path = volume
                        .device_path
                        .clone()
                        .unwrap_or_else(|| volume.label.clone());
                    let mut tasks = vec![Task::done(cosmic::Action::App(
                        Message::SidebarSelectChild {
                            device_path: device_path.clone(),
                        },
                    ))];

                    // Load subvolumes
                    tasks.push(Task::done(cosmic::Action::App(
                        Message::BtrfsLoadSubvolumes {
                            block_path: block_path.clone(),
                            mount_point: mp.clone(),
                        },
                    )));

                    // Load usage info
                    tasks.push(Task::done(cosmic::Action::App(Message::BtrfsLoadUsage {
                        block_path,
                        mount_point: mp,
                    })));

                    return Task::batch(tasks);
                }
            } else {
                // Clear BTRFS state if not a BTRFS volume
                control.btrfs_state = None;
            }
        }

        // Sync with sidebar: select the segment's volume node if it has one
        if let Some(segment) = control.segments.get(index)
            && let Some(vol) = &segment.volume
        {
            let device_path = vol.device_path.clone().unwrap_or_else(|| vol.label.clone());
            return Task::batch(vec![Task::done(cosmic::Action::App(
                Message::SidebarSelectChild { device_path },
            ))]);
        }

        // No volume on this segment (e.g., free space), clear selection
        return Task::batch(vec![Task::done(cosmic::Action::App(
            Message::SidebarClearChildSelection,
        ))]);
    }

    Task::none()
}

pub(super) fn select_volume(
    control: &mut VolumesControl,
    segment_index: usize,
    device_path: String,
    dialog: &Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    if dialog.is_none() {
        let Some(last_index) = control.segments.len().checked_sub(1) else {
            control.selected_segment = 0;
            control.selected_volume = None;
            return Task::none();
        };

        let segment_index = segment_index.min(last_index);
        control.selected_segment = segment_index;
        control.selected_volume = Some(device_path.clone());
        control.detail_tab = DetailTab::VolumeInfo;
        control.usage_state = super::super::state::UsageTabState::default();
        control.segments.iter_mut().for_each(|s| s.state = false);
        if let Some(segment) = control.segments.get_mut(segment_index) {
            segment.state = true;
        }

        // Initialize/update BTRFS state if this segment has a BTRFS volume
        // Also checks through LUKS containers for an inner BTRFS filesystem
        if let Some(segment) = control.segments.get(segment_index)
            && let Some(volume) = &segment.volume
        {
            let btrfs_info = helpers::detect_btrfs_for_volume(&control.volumes, volume);
            tracing::info!(
                "select_volume: segment_index={}, btrfs_detected={}, id_type={}, has_filesystem={}, mount_points={:?}",
                segment_index,
                btrfs_info.is_some(),
                volume.id_type,
                volume.has_filesystem,
                volume.mount_points
            );
            if let Some((mount_point, block_path)) = btrfs_info {
                tracing::info!(
                    "select_volume: Initializing BTRFS state with mount_point={:?}, block_path={}",
                    mount_point,
                    block_path
                );
                control.btrfs_state = Some(BtrfsState::new(
                    mount_point.clone(),
                    Some(block_path.clone()),
                ));

                // If mounted, trigger data loading
                if let Some(mp) = mount_point {
                    let mut tasks = vec![Task::done(cosmic::Action::App(
                        Message::SidebarSelectChild {
                            device_path: device_path.clone(),
                        },
                    ))];

                    // Load subvolumes
                    tasks.push(Task::done(cosmic::Action::App(
                        Message::BtrfsLoadSubvolumes {
                            block_path: block_path.clone(),
                            mount_point: mp.clone(),
                        },
                    )));

                    // Load usage info
                    tasks.push(Task::done(cosmic::Action::App(Message::BtrfsLoadUsage {
                        block_path,
                        mount_point: mp,
                    })));

                    return Task::batch(tasks);
                }
            } else {
                // Clear BTRFS state if not a BTRFS volume
                control.btrfs_state = None;
            }
        }

        // Sync with sidebar: select the corresponding volume in sidebar
        return Task::batch(vec![Task::done(cosmic::Action::App(
            Message::SidebarSelectChild { device_path },
        ))]);
    }

    Task::none()
}
