# BTRFS Feature Recommendations for V2.0+

**Context:** Migration from UDisks2 to btrfsutil unlocks significant new capabilities  
**Audience:** Product planning, UI/UX design  
**Priority System:** ⭐ (1-4 stars, 4 = highest user impact)

---

## Executive Summary

The migration to btrfsutil enables **17+ new metadata fields** and **6+ advanced operations** unavailable in UDisks2. This document recommends **10 high-priority features** to implement, organized by user value and implementation complexity.

### Quick Wins (High Value, Low Complexity)
1. **Read-Only Protection Toggle** ⭐⭐⭐⭐
2. **Creation Timestamps** ⭐⭐⭐
3. **Automatic Snapshot Naming** ⭐⭐⭐

### Major Features (High Value, Medium-High Complexity)
4. **Snapshot Relationship Visualization** ⭐⭐⭐⭐
5. **Default Subvolume Management** ⭐⭐⭐
6. **Quick Snapshot Context Menu** ⭐⭐⭐⭐

### Advanced Features (Medium-High Value, Varies Complexity)
7. **Batch Operations** ⭐⭐⭐
8. **Subvolume Usage Breakdown** ⭐⭐⭐
9. **Search & Filter** ⭐⭐
10. **Deleted Subvolume Cleanup** ⭐⭐

---

## Feature Details

### 1. Read-Only Protection Toggle ⭐⭐⭐⭐
**Priority:** Critical (V2.0 Launch)  
**Complexity:** Low  
**User Impact:** Prevents accidental snapshot modification

#### What It Does
- **Checkbox/Toggle** in subvolume list to set/unset read-only flag
- **Icon indicator** showing current state (lock icon)
- **Confirmation dialog** when making read-only (warns about permanence)
- **Automatic read-only** option when creating snapshots

#### Why Users Need It
- **Snapshot Integrity:** Prevents accidental changes to backups
- **System Recovery:** Ensures restore points remain pristine
- **Regulatory Compliance:** Some sectors require immutable backups
- **Safety:** Typo protection (can't `rm -rf` in read-only subvolume)

#### User Stories
> *"As a system administrator, I want to mark critical snapshots as read-only so my team can't accidentally modify them during incident response."*

> *"As a developer, I want my pre-release snapshots to be immutable so I can always reproduce bugs from that exact state."*

#### UI Mockup
```
[Subvolume List]
├─ @home                    [Make Read-Only ▼]
├─ @home-2026-02-01  🔒     [Make Writable ▼]
├─ @home-2026-02-13         [Make Read-Only ▼]
```

#### Implementation Notes
- Uses `Subvolume::set_ro(bool)`
- Warn if subvolume contains running processes
- Show badge/icon persistently
- Context menu integration

---

### 2. Creation Timestamps ⭐⭐⭐
**Priority:** High (V2.0 Launch)  
**Complexity:** Very Low  
**User Impact:** Essential for snapshot management

#### What It Does
- **Created column** showing when subvolume was created
- **Modified column** showing last change (for non-snapshots)
- **Relative time** display ("2 days ago") with tooltip showing exact time
- **Sort by date** functionality

#### Why Users Need It
- **Snapshot Management:** Identify old snapshots for cleanup
- **Audit Trail:** When was this backup taken?
- **Retention Policies:** "Delete snapshots older than 30 days"
- **Troubleshooting:** Correlate snapshots with system changes

#### User Stories
> *"As a user, I want to see when each snapshot was created so I can find the backup from before my system broke."*

> *"As an admin, I want to sort snapshots by date so I can clean up the oldest ones first."*

#### UI Mockup
```
| Name            | ID | Created        | Modified       | Actions |
|-----------------|----|---------|--------------------|---------|
| @home           | 256| 2 months ago    | 5 minutes ago | ...     |
| @home-backup    | 257| 3 days ago      | 3 days ago    | ...     |
| @home-2026-02-13| 258| 2 hours ago     | 2 hours ago   | ...     |
```

#### Implementation Notes
- Pull from `SubvolumeInfo.otime` (creation) and `ctime` (modification)
- Use `chrono` for relative formatting
- Click timestamp to show full `DateTime` tooltip
- Localize relative time strings

---

### 3. Automatic Snapshot Naming ⭐⭐⭐
**Priority:** High (V2.0 Launch)  
**Complexity:** Low  
**User Impact:** Reduces cognitive load, improves organization

#### What It Does
- **Smart defaults** when creating snapshots
- **Template system** with variables: `{name}`, `{date}`, `{time}`, `{action}`
- **Preview** of generated name before creating
- **Customizable** templates in settings

#### Templates
1. **Timestamped:** `{name}-{date}-{time}` → `@home-2026-02-13-1430`
2. **Action-based:** `{name}-before-{action}` → `@root-before-update`
3. **Sequential:** `{name}-snapshot-{n}` → `@home-snapshot-001`
4. **Date-only:** `{name}-{date}` → `@var-2026-02-13`

#### Why Users Need It
- **No Typing:** One-click snapshot creation
- **Consistency:** All snapshots follow same naming scheme
- **Organization:** Easy to understand structure
- **Automation-Friendly:** Predictable names for scripts

#### User Stories
> *"As a user running a system update, I want snapshots to automatically include 'before-update' in the name so I know what they're for."*

> *"As a power user, I want all my snapshots to use ISO-8601 dates so they sort correctly in scripts."*

#### UI Mockup
```
[Create Snapshot Dialog]
Source: @home
Name: [Template▼] @home-2026-02-13-1435
      └─ Timestamped (default)
         Date Only
         Before Action...
         Sequential
         Custom...

Read-only: [✓]
[Create] [Cancel]
```

#### Implementation Notes
- Store templates in app configuration
- Use `chrono` for date/time formatting
- Validate name (no `/`, must be unique)
- Show preview dynamically as template changes

---

### 4. Snapshot Relationship Visualization ⭐⭐⭐⭐
**Priority:** High (V2.1)  
**Complexity:** Medium  
**User Impact:** Critical for understanding snapshot chains

#### What It Does
- **Tree view** showing parent-child relationships
- **Visual lines** connecting snapshots to originals
- **Snapshot count** badges on parent subvolumes
- **Click to navigate** to parent or children
- **Highlight chain** when hovering over snapshot

#### Why Users Need It
- **Understanding:** See which snapshot came from where
- **Navigation:** Jump to parent/child snapshots quickly
- **Cleanup:** Identify orphaned snapshots
- **Recovery:** Find the right snapshot in a long chain

#### User Stories
> *"As a user with many snapshots, I want to see which snapshots are related so I don't accidentally delete the wrong one."*

> *"As a developer, I want to see my snapshot chain (@main → @feature → @bugfix) so I understand my experimentation history."*

#### UI Mockups

**Option A: Tree View**
```
📁 @home (3 snapshots)
├── 📸 @home-2026-02-01
│   └── 📸 @home-2026-02-01-fixed
├── 📸 @home-2026-02-10
└── 📸 @home-2026-02-13
📁 @var
└── 📸 @var-backup
```

**Option B: Relationship Panel**
```
[Select: @home-2026-02-13]

Relationships:
  Parent:   @home (ID: 256)
            UUID: 1234-5678-...
  
  Children: None
  
  Siblings: @home-2026-02-01 (ID: 257)
            @home-2026-02-10 (ID: 259)
```

#### Implementation Strategy
- Use `SubvolumeInfo.parent_uuid` to match against `SubvolumeInfo.uuid`
- Build graph in memory: `HashMap<Uuid, Vec<BtrfsSubvolume>>`
- Render using tree widget or custom drawing
- Cache relationships (recompute only on refresh)

#### Technical Details
```rust
struct SnapshotGraph {
    // Map from UUID to subvolume
    by_uuid: HashMap<Uuid, BtrfsSubvolume>,
    
    // Map from parent UUID to children
    children: HashMap<Uuid, Vec<Uuid>>,
}

impl SnapshotGraph {
    fn build(subvolumes: &[BtrfsSubvolume]) -> Self {
        let mut graph = Self::default();
        
        for subvol in subvolumes {
            graph.by_uuid.insert(subvol.uuid, subvol.clone());
            
            if let Some(parent_uuid) = subvol.parent_uuid {
                graph.children
                    .entry(parent_uuid)
                    .or_default()
                    .push(subvol.uuid);
            }
        }
        
        graph
    }
    
    fn get_children(&self, subvol: &BtrfsSubvolume) -> Vec<&BtrfsSubvolume> {
        self.children
            .get(&subvol.uuid)
            .map(|uuids| {
                uuids.iter()
                    .filter_map(|uuid| self.by_uuid.get(uuid))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    fn get_parent(&self, subvol: &BtrfsSubvolume) -> Option<&BtrfsSubvolume> {
        subvol.parent_uuid
            .and_then(|uuid| self.by_uuid.get(&uuid))
    }
}
```

---

### 5. Default Subvolume Management ⭐⭐⭐
**Priority:** High (V2.0 Launch)  
**Complexity:** Low-Medium  
**User Impact:** Essential for boot configuration

#### What It Does
- **Badge** showing "DEFAULT" on the default boot subvolume
- **Button** to set any subvolume as default
- **Explanation dialog** warning about boot implications
- **Highlight** in different color

#### Why Users Need It
- **Boot Control:** Change which subvolume mounts at `/`
- **Rollback:** Set older snapshot as default to boot from backup
- **Testing:** Try new configurations without affecting current setup
- **Multi-boot:** Different subvolumes for different distro versions

#### User Stories
> *"As a power user, I want to set yesterday's snapshot as the default subvolume so my system boots from the backup after a failed update."*

> *"As a distro tester, I want to set different subvolumes as default to test different system configurations."*

#### UI Mockup
```
| Name            | ID  | Status  | Actions      |
|-----------------|-----|---------|--------------|
| @             | 256 | DEFAULT🛡️| [Unset▼]     |
| @home           | 257 |         | [Set Default]|
| @home-backup    | 258 |         | [Set Default]|
```

#### Confirmation Dialog
```
⚠️ Change Default Boot Subvolume?

You are about to set "@home-backup" as the default subvolume.

This means:
• Your system will boot using this subvolume
• The subvolume will be mounted at / on next boot
• Your current system state will remain as a snapshot

Current default: @
New default:     @home-backup

This is reversible, but requires a reboot to take effect.

[Change Default] [Cancel]
```

#### Implementation Notes
- Use `Subvolume::get_default()` to find current default
- Use `Subvolume::set_default()` to change
- Requires CAP_SYS_ADMIN (via helper)
- Warning: Only affects BTRFS filesystems, not mountpoints
- Must be on same filesystem

#### Technical Constraints
- Default subvolume is per-filesystem, not per-mountpoint
- Only root filesystem changes affect boot behavior
- May require bootloader reconfiguration (GRUB, systemd-boot)
- Non-root filesystems (e.g., /home) can also have defaults

---

### 6. Quick Snapshot Context Menu ⭐⭐⭐⭐
**Priority:** Critical (V2.0 Launch)  
**Complexity:** Low  
**User Impact:** Dramatically improves UX

#### What It Does
- **Right-click context menu** on subvolumes
- **Common operations** readily available
- **Keyboard shortcuts** for power users
- **Quick snapshot** with one click

#### Menu Items
```
📸 Quick Snapshot Now          Ctrl+T
Properties                     Ctrl+I
─────────────────────────────────────
🔒 Make Read-Only
📌 Set as Default
─────────────────────────────────────
🗑️ Delete                       Del
```

#### Quick Snapshot Behavior
- **Automatic naming** using timestamp template
- **Read-only by default** (checkbox in settings to change)
- **Success notification** with undo option
- **Inline creation** (row appears immediately)

#### Why Users Need It
- **Speed:** Common operations are one click away
- **Discoverability:** Users find features organically
- **Efficiency:** No need to navigate through dialogs
- **Muscle Memory:** Right-click is universal pattern

#### User Stories
> *"As a user about to do something risky, I want to right-click and 'Quick Snapshot Now' so I have a backup in 2 seconds."*

> *"As a power user, I want keyboard shortcuts for snapshot operations so I don't need to use the mouse."*

#### Implementation
```rust
fn subvolume_context_menu<'a>(
    subvol: &'a BtrfsSubvolume,
    is_default: bool,
) -> Element<'a, Message> {
    let menu = widget::column::with_capacity(7)
        .padding(8)
        .spacing(4);
    
    // Quick snapshot
    let menu = menu.push(
        context_menu_item(
            icon("camera-photo-symbolic"),
            fl!("quick-snapshot-now"),
            Message::QuickSnapshot {
                subvolume_id: subvol.id,
            },
            Some("Ctrl+T"),
        )
    );
    
    // Properties
    let menu = menu.push(
        context_menu_item(
            icon("document-properties-symbolic"),
            fl!("properties"),
            Message::ShowProperties {
                subvolume_id: subvol.id,
            },
            Some("Ctrl+I"),
        )
    );
    
    let menu = menu.push(widget::horizontal_rule(1));
    
    // Read-only toggle
    let menu = menu.push(
        context_menu_item(
            if subvol.is_readonly {
                icon("changes-allow-symbolic")
            } else {
                icon("changes-prevent-symbolic")
            },
            if subvol.is_readonly {
                fl!("make-writable")
            } else {
                fl!("make-readonly")
            },
            Message::ToggleReadonly {
                subvolume_id: subvol.id,
            },
            None,
        )
    );
    
    // Set default (if not already)
    if !is_default && !subvol.is_readonly {
        let menu = menu.push(
            context_menu_item(
                icon("emblem-default-symbolic"),
                fl!("set-default"),
                Message::SetDefaultSubvolume {
                    subvolume_id: subvol.id,
                },
                None,
            )
        );
    }
    
    let menu = menu.push(widget::horizontal_rule(1));
    
    // Delete
    let menu = menu.push(
        context_menu_item(
            icon("user-trash-symbolic"),
            fl!("delete"),
            Message::DeleteSubvolume {
                subvolume_id: subvol.id,
            },
            Some("Del"),
        )
        .class(cosmic::theme::Button::Destructive)
    );
    
    widget::popover(
        widget::button::icon(icon("view-more-symbolic"))
            .on_press(Message::ShowContextMenu { subvolume_id: subvol.id })
    )
    .popup(menu)
    .into()
}
```

---

### 7. Batch Operations ⭐⭐⭐
**Priority:** Medium (V2.1)  
**Complexity:** Medium  
**User Impact:** Saves time for bulk management

#### What It Does
- **Checkbox selection mode** in subvolume list
- **Batch action toolbar** appears when items selected
- **Multi-select operations:** delete, snapshot, set read-only
- **Progress indicator** for batch operations

#### Supported Operations
1. **Snapshot All** - Create snapshots of selected subvolumes
2. **Delete All** - Batch delete with single confirmation
3. **Set All Read-Only** - Protect multiple snapshots
4. **Export List** - Save selected subvolumes to file

#### Why Users Need It
- **Efficiency:** Manage 10 snapshots at once, not one-by-one
- **Cleanup:** Delete old snapshots in one action
- **Protection:** Mark all backups read-only together
- **Documentation:** Export snapshot lists for records

#### User Stories
> *"As an admin with 50 old snapshots, I want to select them all and delete in one click rather than 50 individual clicks."*

> *"As a user preparing for updates, I want to snapshot all my important subvolumes (@, @home, @var) at once."*

#### UI Mockup
```
[Select Mode Active]
☑️ Select All | Selected: 3 | [Snapshot All] [Set Read-Only] [Delete] [Cancel]

| ☑️ | Name            | ID  | Created     | Actions |
|----|-----------------|-----|-------------|---------|
| ☑️ | @home-old-1     | 301 | 3 months ago| ...     |
| ☑️ | @home-old-2     | 302 | 3 months ago| ...     |
| ☑️ | @home-old-3     | 303 | 3 months ago| ...     |
| ☐  | @home-current   | 310 | 1 day ago   | ...     |
```

#### Implementation Strategy
```rust
// State
pub struct BtrfsState {
    pub selection_mode: bool,
    pub selected_subvolumes: HashSet<u64>,  // subvolume IDs
    pub batch_operation_progress: Option<BatchProgress>,
}

pub struct BatchProgress {
    pub operation: BatchOperation,
    pub total: usize,
    pub completed: usize,
    pub errors: Vec<(u64, String)>,
}

pub enum BatchOperation {
    Delete,
    Snapshot,
    SetReadonly(bool),
}

// Message handling
Message::EnableSelectionMode => {
    state.selection_mode = true;
    state.selected_subvolumes.clear();
}

Message::ToggleSelection { subvolume_id } => {
    if state.selected_subvolumes.contains(&subvolume_id) {
        state.selected_subvolumes.remove(&subvolume_id);
    } else {
        state.selected_subvolumes.insert(subvolume_id);
    }
}

Message::BatchDelete => {
    // Spawn task for each selected subvolume
    let selected = state.selected_subvolumes.clone();
    Task::future(async move {
        for id in selected {
            // Delete subvolume
        }
    })
}
```

---

### 8. Subvolume Usage Breakdown ⭐⭐⭐
**Priority:** Medium (V2.2)  
**Complexity:** High  
**User Impact:** Essential for capacity planning

#### What It Does
- **Per-subvolume disk usage** (not just filesystem total)
- **Exclusive vs. Referenced** space breakdown
- **Pie chart visualization** of space distribution
- **Enable quota groups** if needed (with warning)

#### Why Users Need It
- **Cleanup:** Identify which snapshots use the most space
- **Planning:** Understand exclusive costs of snapshots
- **Monitoring:** Track usage growth over time
- **Optimization:** Find duplicate data across subvolumes

#### User Stories
> *"As a user with limited disk space, I want to see which snapshots are using the most space so I can delete the biggest ones first."*

> *"As an admin, I want to understand how much exclusive space each subvolume uses so I can budget disk capacity."*

#### Terminology
- **Referenced:** Total data accessible in subvolume (includes shared)
- **Exclusive:** Data unique to this subvolume (deleted if subvolume deleted)
- **Shared:** Data also present in other subvolumes

#### UI Mockup
```
┌─────────────────────────────────────────────────────┐
│ Disk Usage Breakdown                                │
├─────────────────────────────────────────────────────┤
│  @home            ████████████████░░░░░░  12.3 GB   │
│  @home-backup     ██░░░░░░░░░░░░░░░░░░░░   2.1 GB   │
│  @var             ████░░░░░░░░░░░░░░░░░░   4.5 GB   │
│  Other            ██░░░░░░░░░░░░░░░░░░░░   1.8 GB   │
│                                                      │
│  Total: 20.7 GB / 50 GB (41% used)                  │
│                                                      │
│  [○] Pie Chart [●] Bar Chart                        │
│  [ ] Show only exclusive space                      │
└─────────────────────────────────────────────────────┘

| Name          | Referenced | Exclusive | Shared  | % of Total|
|---------------|------------|-----------|---------|-----------|
| @home         | 12.3 GB    | 9.8 GB    | 2.5 GB  | 59%       |
| @home-backup  |  2.1 GB    | 0.3 GB    | 1.8 GB  | 10%       |
| @var          |  4.5 GB    | 4.2 GB    | 0.3 GB  | 22%       |
```

#### Technical Challenge
`libbtrfsutil` **does not provide quota/usage information.** Must fall back to CLI parsing.

#### Implementation Strategy
```rust
pub struct SubvolumeUsage {
    pub subvolume_id: u64,
    pub referenced: u64,  // bytes
    pub exclusive: u64,   // bytes
}

impl BtrfsFilesystem {
    pub async fn get_usage_breakdown(&self) -> Result<Vec<SubvolumeUsage>> {
        // Check if quotas are enabled
        let quotas_enabled = self.check_quotas_enabled().await?;
        
        if !quotas_enabled {
            // Offer to enable
            return Err(anyhow!("Quotas not enabled. Run 'btrfs quota enable' to see usage breakdown."));
        }
        
        // Parse output of `btrfs qgroup show`
        let output = tokio::process::Command::new("btrfs")
            .args(&["qgroup", "show", "-r", &self.mount_point.to_string_lossy()])
            .output()
            .await?;
        
        let stdout = String::from_utf8(output.stdout)?;
        
        // Parse table format:
        // qgroupid          referenced    exclusive  path
        // --------          ----------    ---------  ----
        // 0/256             12.3GiB       9.8GiB     @home
        
        let mut usages = Vec::new();
        
        for line in stdout.lines().skip(2) {  // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let qgroupid = parts[0];
                let referenced_str = parts[1];
                let exclusive_str = parts[2];
                
                // Extract subvolume ID from qgroupid (e.g., "0/256" → 256)
                let subvol_id = qgroupid.split('/').nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .ok_or_else(|| anyhow!("Invalid qgroup ID"))?;
                
                usages.push(SubvolumeUsage {
                    subvolume_id: subvol_id,
                    referenced: parse_size(referenced_str)?,
                    exclusive: parse_size(exclusive_str)?,
                });
            }
        }
        
        Ok(usages)
    }
    
    async fn check_quotas_enabled(&self) -> Result<bool> {
        let output = tokio::process::Command::new("btrfs")
            .args(&["qgroup", "show", &self.mount_point.to_string_lossy()])
            .output()
            .await?;
        
        Ok(output.status.success())
    }
}

fn parse_size(s: &str) -> Result<u64> {
    // Parse "12.3GiB" → bytes
    // Handle KiB, MiB, GiB, TiB
}
```

#### Enabling Quotas
Requires showing an info dialog:

```
⚠️ Enable BTRFS Quotas?

To show per-subvolume disk usage, BTRFS quotas must be enabled.

Impact:
• Slight performance overhead (5-10% on some workloads)
• Additional memory usage for quota tracking
• Can be disabled later with `btrfs quota disable`

Enable quotas on this filesystem?

[Enable] [Cancel] [Learn More...]
```

---

### 9. Search & Filter ⭐⭐
**Priority:** Low-Medium (V2.2)  
**Complexity:** Low  
**User Impact:** Quality of life for large filesystems

#### What It Does
- **Search bar** to filter subvolume list
- **Multiple filter criteria**
- **Real-time filtering** as you type
- **Saved filters** for common queries

#### Filter Types
1. **By Name/Path:** Text matching (case-insensitive)
2. **By Date:** Created before/after date
3. **By Type:** Regular subvolumes vs snapshots
4. **By Flag:** Read-only, default, has children
5. **By Parent UUID:** All snapshots of a specific subvolume

#### Why Users Need It
- **Large Filesystems:** Hundreds of subvolumes are hard to navigate
- **Finding Snapshots:** "Show me all snapshots from last month"
- **Cleanup:** "Show me all read-only snapshots older than 90 days"
- **Relationships:** "Show me all snapshots of @home"

#### User Stories
> *"As a user with 200 snapshots, I want to search by name so I can find specific backups quickly rather than scrolling forever."*

> *"As an admin, I want to filter by creation date so I can find all snapshots older than 3 months for cleanup."*

#### UI Mockup
```
┌─────────────────────────────────────────────────────┐
│ 🔍 [Search subvolumes...         ] [Filters ▼]     │
└─────────────────────────────────────────────────────┘

[Active Filters: Created > 2025-11-01 ✕ | Read-Only ✕]

| Name               | ID  | Created      | Flags  | Actions |
|--------------------|-----|--------------|--------|---------|
| @home-2025-11-15   | 301 | 3 months ago | 🔒     | ...     |
| @home-2025-12-01   | 305 | 2 months ago | 🔒     | ...     |
| @var-2025-11-20    | 310 | 3 months ago | 🔒     | ...     |

Showing 3 of 87 subvolumes [Clear Filters]
```

#### Filters Dialog
```
┌───────────────────────────────────────┐
│ Filter Subvolumes                     │
├───────────────────────────────────────┤
│ Name/Path:    [__________________]    │
│                                       │
│ Created:      [After▼] [2025-01-01]  │
│                                       │
│ Type:         [☑] Regular subvolumes  │
│               [☑] Snapshots           │
│                                       │
│ Flags:        [☐] Read-only only      │
│               [☐] Default only        │
│               [☐] Has children only   │
│                                       │
│ Parent UUID:  [__________________]    │
│               (show snapshots of...)  │
│                                       │
│               [Apply] [Clear] [Close] │
└───────────────────────────────────────┘
```

#### Implementation
```rust
#[derive(Default, Clone)]
pub struct SubvolumeFilter {
    pub name_contains: Option<String>,
    pub created_after: Option<DateTime<Local>>,
    pub created_before: Option<DateTime<Local>>,
    pub show_regular: bool,
    pub show_snapshots: bool,
    pub readonly_only: bool,
    pub default_only: bool,
    pub parent_uuid: Option<Uuid>,
}

impl SubvolumeFilter {
    pub fn matches(&self, subvol: &BtrfsSubvolume) -> bool {
        // Name filter
        if let Some(name) = &self.name_contains {
            if !subvol.path.to_string_lossy().to_lowercase()
                .contains(&name.to_lowercase())
            {
                return false;
            }
        }
        
        // Date filters
        if let Some(after) = self.created_after {
            if subvol.created < after {
                return false;
            }
        }
        
        if let Some(before) = self.created_before {
            if subvol.created > before {
                return false;
            }
        }
        
        // Type filter
        let is_snapshot = subvol.parent_uuid.is_some();
        if is_snapshot && !self.show_snapshots {
            return false;
        }
        if !is_snapshot && !self.show_regular {
            return false;
        }
        
        // Flag filters
        if self.readonly_only && !subvol.is_readonly {
            return false;
        }
        
        if self.default_only && !subvol.is_default {
            return false;
        }
        
        // Parent UUID filter
        if let Some(parent) = self.parent_uuid {
            if subvol.parent_uuid != Some(parent) {
                return false;
            }
        }
        
        true
    }
}

// In view rendering:
let filtered_subvolumes: Vec<_> = subvolumes.iter()
    .filter(|s| state.filter.matches(s))
    .collect();

// Show count
widget::text(format!(
    "Showing {} of {} subvolumes",
    filtered_subvolumes.len(),
    subvolumes.len()
))
```

---

### 10. Deleted Subvolume Cleanup ⭐⭐
**Priority:** Low-Medium (V2.0 Launch)  
**Complexity:** Low  
**User Impact:** Reclaim space, housekeeping

#### What It Does
- **List deleted subvolumes** (pending cleanup)
- **Show how much space** would be reclaimed
- **One-click cleanup** button
- **Collapsible section** in main view

#### Why Users Need It
- **Space Reclamation:** Deleted subvolumes still occupy space until cleaned
- **Housekeeping:** Regular maintenance task
- **Visibility:** Users don't know about deleted subvolume state
- **Automation:** Could be scheduled in future

#### User Stories
> *"As a user who deleted snapshots, I want to see that they're still using space so I can complete the cleanup."*

> *"As an admin, I want a one-click button to reclaim all space from deleted subvolumes rather than running CLI commands."*

#### UI Mockup
```
┌─────────────────────────────────────────────────────┐
│ ⚠️ Deleted Subvolumes (Click to expand)       [▼]   │
└─────────────────────────────────────────────────────┘

[Expanded:]
┌─────────────────────────────────────────────────────┐
│ ⚠️ Deleted Subvolumes                          [▲]   │
├─────────────────────────────────────────────────────┤
│ 3 subvolumes pending cleanup (~2.4 GB to reclaim)   │
│                                                      │
│ • ID 297: @home-old-1 (deleted 2 days ago)          │
│ • ID 298: @home-old-2 (deleted 2 days ago)          │
│ • ID 301: @var-old (deleted 1 week ago)             │
│                                                      │
│             [Clean Up Now]  [Learn More...]         │
└─────────────────────────────────────────────────────┘
```

#### Technical Background
BTRFS uses **generations** and **reference counting.** When you delete a subvolume:
1. It's marked for deletion but not immediately removed
2. Data remains until all references are cleared
3. A cleanup pass reclaims the space

`Subvolume::deleted()` lists subvolumes in this pending state.

#### Implementation
```rust
impl BtrfsFilesystem {
    pub async fn list_deleted_subvolumes(&self) -> Result<Vec<BtrfsSubvolume>> {
        // Get filesystem root
        let root = tokio::task::spawn_blocking({
            let mount_point = self.mount_point.clone();
            move || {
                btrfsutil::subvolume::Subvolume::try_from(mount_point.as_path())
            }
        })
        .await??;
        
        // List deleted subvolumes
        let deleted_iter = tokio::task::spawn_blocking(move || {
            btrfsutil::subvolume::Subvolume::deleted(&root)
        })
        .await??;
        
        // Convert to our type
        let mut deleted = Vec::new();
        for subvol in deleted_iter {
            let subvol = subvol?;
            let info = subvol.info()?;
            deleted.push(BtrfsSubvolume::from(info));
        }
        
        Ok(deleted)
    }
    
    pub async fn cleanup_deleted(&self) -> Result<()> {
        // Trigger BTRFS cleanup
        // This is done via `btrfs subvolume sync` command
        tokio::process::Command::new("btrfs")
            .args(&["subvolume", "sync", &self.mount_point.to_string_lossy()])
            .output()
            .await?;
        
        Ok(())
    }
}
```

#### Explanation Dialog
When user clicks "Learn More...":

```
What are Deleted Subvolumes?

When you delete a BTRFS subvolume, it's not immediately removed.
Instead, it's marked for deletion and cleaned up later during a
sync operation.

This is normal BTRFS behavior. The space is not lost, just not yet
reclaimed.

Clicking "Clean Up Now" runs `btrfs subvolume sync` which:
• Waits for all references to be cleared
• Reclaims disk space
• May take a few seconds to minutes

This is safe and recommended as regular maintenance.

[Got It]
```

---

## Implementation Priority Matrix

### V2.0 Launch (Must Have)
| Feature | Priority | Complexity | Weeks |
|---------|----------|------------|-------|
| Read-Only Toggle | ⭐⭐⭐⭐ | Low | 0.5 |
| Creation Timestamps | ⭐⭐⭐ | Very Low | 0.3 |
| Default Subvolume | ⭐⭐⭐ | Low-Medium | 0.5 |
| Context Menu | ⭐⭐⭐⭐ | Low | 0.5 |
| Deleted Cleanup | ⭐⭐ | Low | 0.3 |
| Automatic Naming | ⭐⭐⭐ | Low | 0.5 |
| **Total** | | | **~2.6 weeks** |

### V2.1 Features (High Value)
| Feature | Priority | Complexity | Weeks |
|---------|----------|------------|-------|
| Snapshot Relationships | ⭐⭐⭐⭐ | Medium | 1.5 |
| Batch Operations | ⭐⭐⭐ | Medium | 1.0 |
| **Total** | | | **~2.5 weeks** |

### V2.2 Features (Nice to Have)
| Feature | Priority | Complexity | Weeks |
|---------|----------|------------|-------|
| Usage Breakdown | ⭐⭐⭐ | High | 2.0 |
| Search & Filter | ⭐⭐ | Low | 0.5 |
| **Total** | | | **~2.5 weeks** |

**Total Implementation Time:** ~7-8 weeks for all features

---

## Competitive Analysis

### vs. GNOME Disks
- ✅ **We have:** Snapshot relationships, batch operations, advanced naming
- ❌ **They have:** (No BTRFS support beyond basic partitioning)

### vs. Timeshift
- ✅ **We have:** Calendar-based scheduling, systemd integration
- ❌ **We lack:** Scheduling (planned V3), pre/post hooks

### vs. Snapper
- ✅ **We have:** GUI, user-friendly
- ❌ **We lack:** Automated cleanup policies (planned V2.2)

### vs. `btrfs` CLI
- ✅ **We have:** Visual representation, user-friendly
- ❌ **We lack:** Advanced send/receive (planned V3)

**Our Niche:** Best-in-class GUI for BTRFS subvolume management with snapshot visualization

---

## User Persona Alignment

### Persona 1: Casual User
**Needs:** Easy backups before system changes  
**Features:**
- ⭐⭐⭐⭐ Quick snapshot context menu
- ⭐⭐⭐ Automatic naming
- ⭐⭐⭐ Read-only protection

### Persona 2: Power User
**Needs:** Advanced snapshot management, scripting support  
**Features:**
- ⭐⭐⭐⭐ Snapshot relationships
- ⭐⭐⭐ Batch operations
- ⭐⭐⭐ Usage breakdown
- ⭐⭐ Search & filter

### Persona 3: System Administrator
**Needs:** Multi-system management, compliance  
**Features:**
- ⭐⭐⭐ Default subvolume (boot control)
- ⭐⭐⭐ Usage breakdown
- ⭐⭐⭐ Batch operations
- ⭐⭐ Deleted cleanup

---

## Conclusion

The migration to **btrfsutil** enables **10 high-impact features** that transform Cosmic Disks from a basic BTRFS tool into a **best-in-class subvolume manager.**

### Recommended Phases
1. **V2.0:** 6 core features (2-3 weeks) - Quick wins, immediate value
2. **V2.1:** 2 advanced features (2-3 weeks) - Relationship graph, batch ops
3. **V2.2:** 2 nice-to-have features (2-3 weeks) - Usage, search/filter

**Total:** ~7-8 weeks to achieve feature parity with Timeshift/Snapper + unique GUI advantages

### Competitive Advantage
- **Only GUI tool** with snapshot relationship visualization
- **Most user-friendly** BTRFS subvolume manager
- **Integrated** with COSMIC desktop environment
- **Modern** Rust/libcosmic implementation

🚀 **Ready to implement immediately after migration completes**
