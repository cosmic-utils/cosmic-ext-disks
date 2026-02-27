use std::collections::{BTreeMap, HashMap};

use storage_types::{
    LogicalCapabilities, LogicalEntity, LogicalEntityKind, LogicalMember, LogicalOperation,
};
use zbus::{Connection, fdo::ObjectManagerProxy, zvariant::OwnedValue};

use crate::DiskError;

const MDRAID_IFACE: &str = "org.freedesktop.UDisks2.MDRaid";
const BLOCK_IFACE: &str = "org.freedesktop.UDisks2.Block";

#[derive(Debug, Clone)]
struct MdRaidRaw {
    object_path: String,
    uuid: Option<String>,
    name: Option<String>,
    level: Option<String>,
    size: u64,
    degraded: u32,
    running: Option<bool>,
}

fn as_string(value: &OwnedValue) -> Option<String> {
    String::try_from(value.clone()).ok()
}

fn as_u64(value: &OwnedValue) -> Option<u64> {
    if let Ok(parsed) = u64::try_from(value.clone()) {
        Some(parsed)
    } else {
        u32::try_from(value.clone()).ok().map(u64::from)
    }
}

fn as_u32(value: &OwnedValue) -> Option<u32> {
    u32::try_from(value.clone()).ok()
}

fn as_bool(value: &OwnedValue) -> Option<bool> {
    bool::try_from(value.clone()).ok()
}

fn parse_mdraid_raw(object_path: &str, properties: &HashMap<String, OwnedValue>) -> MdRaidRaw {
    MdRaidRaw {
        object_path: object_path.to_string(),
        uuid: properties.get("UUID").and_then(as_string),
        name: properties.get("Name").and_then(as_string),
        level: properties.get("Level").and_then(as_string),
        size: properties.get("Size").and_then(as_u64).unwrap_or(0),
        degraded: properties.get("Degraded").and_then(as_u32).unwrap_or(0),
        running: properties.get("Running").and_then(as_bool),
    }
}

fn map_mdraid_entities(
    arrays: &[MdRaidRaw],
    member_map: &HashMap<String, Vec<LogicalMember>>,
) -> Vec<LogicalEntity> {
    let mut entities = Vec::with_capacity(arrays.len());

    for array in arrays {
        let array_id = format!("mdraid:{}", array.object_path);
        let health_status = if array.degraded > 0 {
            Some("degraded".to_string())
        } else {
            Some("ok".to_string())
        };

        let mut metadata = BTreeMap::new();
        if let Some(level) = &array.level {
            metadata.insert("level".to_string(), level.clone());
        }
        if let Some(running) = array.running {
            metadata.insert("running".to_string(), running.to_string());
        }

        entities.push(LogicalEntity {
            id: array_id,
            kind: LogicalEntityKind::MdRaidArray,
            name: array
                .name
                .clone()
                .unwrap_or_else(|| array.object_path.clone()),
            uuid: array.uuid.clone(),
            parent_id: None,
            device_path: None,
            size_bytes: array.size,
            used_bytes: None,
            free_bytes: None,
            health_status,
            progress_fraction: None,
            members: member_map
                .get(&array.object_path)
                .cloned()
                .unwrap_or_default(),
            capabilities: LogicalCapabilities {
                supported: vec![
                    LogicalOperation::Create,
                    LogicalOperation::Delete,
                    LogicalOperation::Start,
                    LogicalOperation::Stop,
                    LogicalOperation::AddMember,
                    LogicalOperation::RemoveMember,
                    LogicalOperation::Check,
                    LogicalOperation::Repair,
                ],
                blocked: vec![],
            },
            metadata,
        });
    }

    entities
}

/// Discover mdraid entities from UDisks ObjectManager.
pub async fn discover_mdraid_entities(
    connection: &Connection,
) -> Result<Vec<LogicalEntity>, DiskError> {
    let proxy = ObjectManagerProxy::builder(connection)
        .destination("org.freedesktop.UDisks2")
        .map_err(|error| DiskError::DBusError(error.to_string()))?
        .path("/org/freedesktop/UDisks2")
        .map_err(|error| DiskError::DBusError(error.to_string()))?
        .build()
        .await
        .map_err(|error| DiskError::DBusError(error.to_string()))?;

    let managed = proxy
        .get_managed_objects()
        .await
        .map_err(|error| DiskError::DBusError(error.to_string()))?;

    let mut arrays = Vec::<MdRaidRaw>::new();
    let mut members_by_array = HashMap::<String, Vec<LogicalMember>>::new();

    for (object_path, interfaces) in &managed {
        if let Some(properties) = interfaces.get(MDRAID_IFACE) {
            arrays.push(parse_mdraid_raw(object_path.as_str(), properties));
        }
    }

    for (object_path, interfaces) in &managed {
        let Some(block_properties) = interfaces.get(BLOCK_IFACE) else {
            continue;
        };

        let Some(member_of) = block_properties.get("MDRaidMember").and_then(as_string) else {
            continue;
        };

        if member_of == "/" {
            continue;
        }

        let preferred_device = block_properties.get("PreferredDevice").and_then(as_string);

        members_by_array
            .entry(member_of)
            .or_default()
            .push(LogicalMember {
                id: format!("mdraid-member:{}", object_path.as_str()),
                name: object_path.as_str().to_string(),
                device_path: preferred_device,
                role: Some("member".to_string()),
                state: None,
                size_bytes: block_properties.get("Size").and_then(as_u64),
            });
    }

    Ok(map_mdraid_entities(&arrays, &members_by_array))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mdraid_raw_properties() {
        let properties = HashMap::from([
            ("Size".to_string(), OwnedValue::from(4096_u64)),
            ("Degraded".to_string(), OwnedValue::from(0_u32)),
            ("Running".to_string(), OwnedValue::from(true)),
        ]);

        let raw = parse_mdraid_raw("/org/freedesktop/UDisks2/mdraid/md0", &properties);
        assert_eq!(raw.uuid, None);
        assert_eq!(raw.name, None);
        assert_eq!(raw.level, None);
        assert_eq!(raw.size, 4096);
        assert_eq!(raw.degraded, 0);
        assert_eq!(raw.running, Some(true));
    }

    #[test]
    fn maps_mdraid_entities_with_health() {
        let arrays = vec![MdRaidRaw {
            object_path: "/org/freedesktop/UDisks2/mdraid/md0".to_string(),
            uuid: Some("abc-123".to_string()),
            name: Some("md0".to_string()),
            level: Some("raid1".to_string()),
            size: 4096,
            degraded: 1,
            running: Some(true),
        }];

        let members = HashMap::from([(
            "/org/freedesktop/UDisks2/mdraid/md0".to_string(),
            vec![LogicalMember {
                id: "m1".to_string(),
                name: "member".to_string(),
                device_path: Some("/dev/sdb1".to_string()),
                role: Some("member".to_string()),
                state: Some("active".to_string()),
                size_bytes: Some(2048),
            }],
        )]);

        let entities = map_mdraid_entities(&arrays, &members);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].kind, LogicalEntityKind::MdRaidArray);
        assert_eq!(entities[0].health_status.as_deref(), Some("degraded"));
        assert_eq!(entities[0].members.len(), 1);
    }
}
