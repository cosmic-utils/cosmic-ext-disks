use cosmic::Task;

use crate::app::Message;
use crate::ui::dialogs::state::ShowDialog;

use super::super::VolumesControl;

pub(super) fn segment_selected(
    control: &mut VolumesControl,
    index: usize,
    dialog: &Option<ShowDialog>,
) -> Task<cosmic::Action<Message>> {
    if dialog.is_none() {
        let Some(last_index) = control.segments.len().checked_sub(1) else {
            control.selected_segment = 0;
            control.selected_volume = None;
            return Task::none();
        };

        let index = index.min(last_index);
        control.selected_segment = index;
        control.selected_volume = None;
        control.segments.iter_mut().for_each(|s| s.state = false);
        if let Some(segment) = control.segments.get_mut(index) {
            segment.state = true;
        }
    }

    Task::none()
}

pub(super) fn select_volume(
    control: &mut VolumesControl,
    segment_index: usize,
    object_path: String,
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
        control.selected_volume = Some(object_path);
        control.segments.iter_mut().for_each(|s| s.state = false);
        if let Some(segment) = control.segments.get_mut(segment_index) {
            segment.state = true;
        }
    }

    Task::none()
}

pub(super) fn toggle_show_reserved(
    control: &mut VolumesControl,
    show_reserved: bool,
) -> Task<cosmic::Action<Message>> {
    control.set_show_reserved(show_reserved);
    Task::none()
}
