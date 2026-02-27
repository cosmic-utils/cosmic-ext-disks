//! Logical storage topology models.
//!
//! These models represent cross-device and logical storage entities that are not
//! tied to a single disk partition map (e.g. LVM VGs/LVs, MD RAID arrays, and
//! multi-device BTRFS filesystems).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Kind of logical entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalEntityKind {
    LvmVolumeGroup,
    LvmLogicalVolume,
    LvmPhysicalVolume,
    MdRaidArray,
    MdRaidMember,
    BtrfsFilesystem,
    BtrfsDevice,
    BtrfsSubvolume,
}

/// Operation that can be requested on a logical entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicalOperation {
    Create,
    Delete,
    Resize,
    AddMember,
    RemoveMember,
    Activate,
    Deactivate,
    Start,
    Stop,
    Check,
    Repair,
    SetLabel,
    SetDefaultSubvolume,
}

/// Blocked reason for a specific operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalBlockedReason {
    pub operation: LogicalOperation,
    pub reason: String,
}

/// Capability set exposed by service for UI gating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LogicalCapabilities {
    pub supported: Vec<LogicalOperation>,
    pub blocked: Vec<LogicalBlockedReason>,
}

impl LogicalCapabilities {
    pub fn is_supported(&self, operation: LogicalOperation) -> bool {
        self.supported.contains(&operation)
    }

    pub fn blocked_reason(&self, operation: LogicalOperation) -> Option<&str> {
        self.blocked
            .iter()
            .find(|entry| entry.operation == operation)
            .map(|entry| entry.reason.as_str())
    }

    pub fn is_allowed(&self, operation: LogicalOperation) -> bool {
        self.is_supported(operation) && self.blocked_reason(operation).is_none()
    }
}

/// Member entry under a logical aggregate/root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalMember {
    pub id: String,
    pub name: String,
    pub device_path: Option<String>,
    pub role: Option<String>,
    pub state: Option<String>,
    pub size_bytes: Option<u64>,
}

/// Canonical logical entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalEntity {
    /// Stable ID used for parent/member linking and UI selection.
    pub id: String,
    pub kind: LogicalEntityKind,
    pub name: String,
    pub uuid: Option<String>,
    pub parent_id: Option<String>,
    pub device_path: Option<String>,
    pub size_bytes: u64,
    pub used_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
    pub health_status: Option<String>,
    /// 0.0..=1.0 when a background operation reports progress.
    pub progress_fraction: Option<ProgressRatio>,
    pub members: Vec<LogicalMember>,
    pub capabilities: LogicalCapabilities,
    /// Provider-specific fields that are still useful for display.
    pub metadata: BTreeMap<String, String>,
}

impl LogicalEntity {
    pub fn inferred_used_bytes(&self) -> Option<u64> {
        if let Some(used_bytes) = self.used_bytes {
            return Some(used_bytes);
        }

        self.free_bytes
            .map(|free_bytes| self.size_bytes.saturating_sub(free_bytes))
    }

    pub fn inferred_free_bytes(&self) -> Option<u64> {
        if let Some(free_bytes) = self.free_bytes {
            return Some(free_bytes);
        }

        self.used_bytes
            .map(|used_bytes| self.size_bytes.saturating_sub(used_bytes))
    }
}

/// Newtype ratio for progress to keep values bounded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressRatio(u32);

impl ProgressRatio {
    pub fn from_fraction(fraction: f64) -> Self {
        let clamped = fraction.clamp(0.0, 1.0);
        Self((clamped * 10_000.0).round() as u32)
    }

    pub fn as_fraction(self) -> f64 {
        self.0 as f64 / 10_000.0
    }
}

/// Aggregate summary for a set of logical entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LogicalAggregateSummary {
    pub entity_count: usize,
    pub total_size_bytes: u64,
    pub total_used_bytes: u64,
    pub total_free_bytes: u64,
    pub degraded_count: usize,
}

/// Build a safe aggregate summary for selected logical entities.
pub fn summarize_entities(entities: &[LogicalEntity]) -> LogicalAggregateSummary {
    let mut summary = LogicalAggregateSummary {
        entity_count: entities.len(),
        ..Default::default()
    };

    for entity in entities {
        summary.total_size_bytes = summary.total_size_bytes.saturating_add(entity.size_bytes);

        if let Some(used_bytes) = entity.inferred_used_bytes() {
            summary.total_used_bytes = summary.total_used_bytes.saturating_add(used_bytes);
        }

        if let Some(free_bytes) = entity.inferred_free_bytes() {
            summary.total_free_bytes = summary.total_free_bytes.saturating_add(free_bytes);
        }

        let is_degraded = entity
            .health_status
            .as_deref()
            .is_some_and(|status| status.eq_ignore_ascii_case("degraded"));
        if is_degraded {
            summary.degraded_count += 1;
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entity(id: &str, kind: LogicalEntityKind) -> LogicalEntity {
        LogicalEntity {
            id: id.to_string(),
            kind,
            name: id.to_string(),
            uuid: Some(format!("uuid-{id}")),
            parent_id: None,
            device_path: Some(format!("/dev/{id}")),
            size_bytes: 100,
            used_bytes: Some(40),
            free_bytes: Some(60),
            health_status: Some("ok".to_string()),
            progress_fraction: Some(ProgressRatio::from_fraction(0.5)),
            members: vec![],
            capabilities: LogicalCapabilities {
                supported: vec![LogicalOperation::Resize, LogicalOperation::Delete],
                blocked: vec![LogicalBlockedReason {
                    operation: LogicalOperation::Delete,
                    reason: "has dependents".to_string(),
                }],
            },
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn serde_roundtrip_logical_entity() {
        let entity = sample_entity("vg0", LogicalEntityKind::LvmVolumeGroup);

        let json = serde_json::to_string(&entity).expect("serialize entity");
        let parsed: LogicalEntity = serde_json::from_str(&json).expect("deserialize entity");

        assert_eq!(parsed, entity);
    }

    #[test]
    fn capabilities_and_blocked_reason_work() {
        let entity = sample_entity("lv0", LogicalEntityKind::LvmLogicalVolume);

        assert!(entity.capabilities.is_supported(LogicalOperation::Resize));
        assert!(entity.capabilities.is_supported(LogicalOperation::Delete));
        assert!(!entity.capabilities.is_allowed(LogicalOperation::Delete));
        assert_eq!(
            entity.capabilities.blocked_reason(LogicalOperation::Delete),
            Some("has dependents")
        );
    }

    #[test]
    fn member_parent_linking_is_representable() {
        let mut root = sample_entity("md0", LogicalEntityKind::MdRaidArray);
        root.members = vec![LogicalMember {
            id: "md0-member-sdb1".to_string(),
            name: "sdb1".to_string(),
            device_path: Some("/dev/sdb1".to_string()),
            role: Some("in_sync".to_string()),
            state: Some("active".to_string()),
            size_bytes: Some(100),
        }];

        let mut child = sample_entity("md0-member-sdb1", LogicalEntityKind::MdRaidMember);
        child.parent_id = Some(root.id.clone());

        assert_eq!(child.parent_id.as_deref(), Some(root.id.as_str()));
        assert_eq!(root.members.len(), 1);
        assert_eq!(root.members[0].device_path.as_deref(), Some("/dev/sdb1"));
    }

    #[test]
    fn aggregate_summary_is_calculated_correctly() {
        let mut first = sample_entity("a", LogicalEntityKind::LvmVolumeGroup);
        first.size_bytes = 100;
        first.used_bytes = Some(25);
        first.free_bytes = Some(75);

        let mut second = sample_entity("b", LogicalEntityKind::MdRaidArray);
        second.size_bytes = 300;
        second.used_bytes = Some(200);
        second.free_bytes = Some(100);
        second.health_status = Some("degraded".to_string());

        let summary = summarize_entities(&[first, second]);

        assert_eq!(summary.entity_count, 2);
        assert_eq!(summary.total_size_bytes, 400);
        assert_eq!(summary.total_used_bytes, 225);
        assert_eq!(summary.total_free_bytes, 175);
        assert_eq!(summary.degraded_count, 1);
    }

    #[test]
    fn progress_ratio_clamps_fraction() {
        let below = ProgressRatio::from_fraction(-5.0).as_fraction();
        let above = ProgressRatio::from_fraction(42.0).as_fraction();

        assert_eq!(below, 0.0);
        assert_eq!(above, 1.0);
    }
}
