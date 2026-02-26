use storage_types::{LogicalEntity, LogicalOperation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogicalDetailTab {
    #[default]
    Overview,
    Members,
    Operations,
    Btrfs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingLogicalAction {
    pub entity_id: String,
    pub operation: LogicalOperation,
}

#[derive(Debug, Default, Clone)]
pub struct LogicalState {
    pub entities: Vec<LogicalEntity>,
    pub selected_entity_id: Option<String>,
    pub last_error: Option<String>,
    pub detail_tab: LogicalDetailTab,
    pub pending_action: Option<PendingLogicalAction>,
    pub operation_status: Option<String>,
}

impl LogicalState {
    pub fn selected_entity(&self) -> Option<&LogicalEntity> {
        let selected_id = self.selected_entity_id.as_deref()?;
        self.entities.iter().find(|entity| entity.id == selected_id)
    }

    pub fn set_entities(&mut self, entities: Vec<LogicalEntity>) {
        self.entities = entities;

        if let Some(selected) = &self.selected_entity_id
            && !self.entities.iter().any(|entity| entity.id == *selected)
        {
            self.selected_entity_id = None;
        }

        if let Some(pending) = &self.pending_action
            && !self
                .entities
                .iter()
                .any(|entity| entity.id == pending.entity_id)
        {
            self.pending_action = None;
        }
    }

    pub fn select_entity(&mut self, entity_id: String) {
        self.selected_entity_id = Some(entity_id);
        self.pending_action = None;
        self.operation_status = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage_types::{LogicalCapabilities, LogicalEntityKind};

    fn entity(id: &str) -> LogicalEntity {
        LogicalEntity {
            id: id.to_string(),
            kind: LogicalEntityKind::LvmLogicalVolume,
            name: id.to_string(),
            uuid: None,
            parent_id: None,
            device_path: Some(format!("/dev/{id}")),
            size_bytes: 1024,
            used_bytes: None,
            free_bytes: None,
            health_status: None,
            progress_fraction: None,
            members: vec![],
            capabilities: LogicalCapabilities::default(),
            metadata: Default::default(),
        }
    }

    #[test]
    fn set_entities_clears_stale_selection_and_pending_action() {
        let mut state = LogicalState {
            selected_entity_id: Some("a".to_string()),
            pending_action: Some(PendingLogicalAction {
                entity_id: "a".to_string(),
                operation: LogicalOperation::Delete,
            }),
            ..Default::default()
        };

        state.set_entities(vec![entity("b")]);

        assert_eq!(state.selected_entity_id, None);
        assert_eq!(state.pending_action, None);
    }

    #[test]
    fn select_entity_resets_operation_feedback() {
        let mut state = LogicalState {
            operation_status: Some("failed".to_string()),
            pending_action: Some(PendingLogicalAction {
                entity_id: "a".to_string(),
                operation: LogicalOperation::Delete,
            }),
            ..Default::default()
        };

        state.select_entity("b".to_string());

        assert_eq!(state.selected_entity_id.as_deref(), Some("b"));
        assert_eq!(state.operation_status, None);
        assert_eq!(state.pending_action, None);
    }
}
