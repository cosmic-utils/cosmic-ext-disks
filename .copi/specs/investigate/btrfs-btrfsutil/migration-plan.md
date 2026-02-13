# Full Migration Plan: UDisks2 → btrfsutil (Destructive)

**Branch Strategy:** `investigate/btrfs-btrfsutil` → `refactor/btrfs-btrfsutil` → merge to `main`  
**Target Version:** V2.0.0 (breaking change)  
**Approach:** Clean break, no compatibility layer  
**Timeline:** 4-6 weeks full-time equivalent

---

## Migration Philosophy

### Clean Break Principles
1. **No Legacy Code:** Remove all UDisks2 BTRFS code completely
2. **No Feature Flags:** btrfsutil is the only implementation
3. **No Compatibility Layer:** UI/API changes accepted if beneficial
4. **Modern Rust:** Use latest async patterns, proper error types
5. **Type Safety:** Leverage Rust type system for correctness

### Why Destructive?
- V2.0 allows breaking changes
- Simpler codebase (no dual implementations)
- Better maintainability long-term
- Enables architectural improvements
- Users upgrading expect new features

---

## System Architecture Changes

### Current Architecture
```
┌─────────────────────────────────────────────────────────────┐
│ UI Layer (disks-ui)                                        │
│  - volumes/view.rs: Render BTRFS tab                       │
│  - volumes/update.rs: Handle BTRFS messages                │
│  - btrfs/view.rs: Subvolume list rendering                 │
│  - btrfs/state.rs: BTRFS state management                  │
└─────────────────┬───────────────────────────────────────────┘
                  │ Messages (async Task)
┌─────────────────▼───────────────────────────────────────────┐
│ Business Logic (disks-ui/src/ui/app/update/btrfs.rs)      │
│  - Handle user actions                                     │
│  - Spawn async operations                                  │
└─────────────────┬───────────────────────────────────────────┘
                  │ Async calls
┌─────────────────▼───────────────────────────────────────────┐
│ D-Bus Wrapper (disks-dbus/src/disks/btrfs.rs)             │
│  - BtrfsFilesystem struct                                  │
│  - zbus proxy creation                                     │
│  - D-Bus method calls                                      │
└─────────────────┬───────────────────────────────────────────┘
                  │ D-Bus IPC
┌─────────────────▼───────────────────────────────────────────┐
│ UDisks2 Daemon (udisks2)                                   │
│  - org.freedesktop.UDisks2.Filesystem.BTRFS                │
│  - Polkit authorization                                    │
│  - Privileged operations                                   │
└─────────────────┬───────────────────────────────────────────┘
                  │ ioctl syscalls
┌─────────────────▼───────────────────────────────────────────┐
│ Kernel (BTRFS FS)                                          │
└─────────────────────────────────────────────────────────────┘
```

### New Architecture
```
┌─────────────────────────────────────────────────────────────┐
│ UI Layer (disks-ui)                                        │
│  - volumes/view.rs: Render BTRFS tab (enhanced)            │
│  - volumes/update.rs: Handle BTRFS messages (expanded)     │
│  - btrfs/view.rs: Rich subvolume rendering                 │
│  - btrfs/state.rs: Extended state with metadata           │
│  - btrfs/properties.rs: NEW - Subvolume properties dialog  │
└─────────────────┬───────────────────────────────────────────┘
                  │ Messages (async Task)
┌─────────────────▼───────────────────────────────────────────┐
│ Business Logic (disks-ui/src/ui/app/update/btrfs.rs)      │
│  - Handle user actions                                     │
│  - Spawn blocking operations via tokio::spawn_blocking     │
│  - Rich error handling                                     │
└─────────────────┬───────────────────────────────────────────┘
                  │ Blocking calls (tokio::spawn_blocking)
┌─────────────────▼───────────────────────────────────────────┐
│ BTRFS Wrapper (disks-dbus/src/disks/btrfs_native.rs) NEW  │
│  - BtrfsFilesystem struct (new implementation)             │
│  - Privilege helper integration                            │
│  - Error type conversions                                  │
│  - Async-friendly blocking wrappers                        │
└─────────────────┬───────────────────────────────────────────┘
                  │ Rust FFI
┌─────────────────▼───────────────────────────────────────────┐
│ btrfsutil crate                                            │
│  - Subvolume struct                                        │
│  - SubvolumeInfo with rich metadata                        │
│  - QgroupInherit                                           │
│  - Safe wrappers for libbtrfsutil                          │
└─────────────────┬───────────────────────────────────────────┘
                  │ C FFI
┌─────────────────▼───────────────────────────────────────────┐
│ libbtrfsutil.so                                            │
│  - Official BTRFS library from btrfs-progs                 │
└─────────────────┬───────────────────────────────────────────┘
                  │ ioctl syscalls (via helper)
┌─────────────────▼───────────────────────────────────────────┐
│ Privilege Helper (cosmic-ext-disks-btrfs-helper) NEW      │
│  - Minimal privileged binary                               │
│  - Polkit integration                                      │
│  - Command-line interface                                  │
│  - CAP_SYS_ADMIN operations only                           │
└─────────────────┬───────────────────────────────────────────┘
                  │ ioctl syscalls
┌─────────────────▼───────────────────────────────────────────┐
│ Kernel (BTRFS FS)                                          │
└─────────────────────────────────────────────────────────────┘
```

### Key Differences
1. **No D-Bus layer** - Direct library calls
2. **Privilege helper binary** - Separate process for elevated operations
3. **Blocking operations** - Wrapped in `tokio::spawn_blocking()`
4. **Richer state** - More metadata in UI state
5. **New UI components** - Properties dialog, advanced features

---

## File-by-File Migration Plan

### Phase 1: Core Library Integration

#### 1.1 Add Dependencies
**File:** `disks-dbus/Cargo.toml`

```toml
# ADD:
btrfsutil = "0.2.0"
uuid = { version = "1.10", features = ["serde"] }
chrono = { version = "0.4", features = ["serde"] }

# REMOVE (if not used elsewhere):
# udisks2-btrfs module dependency (system-level)
```

#### 1.2 Create New BTRFS Module
**New File:** `disks-dbus/src/disks/btrfs_native.rs`

**Purpose:** Complete rewrite using btrfsutil

**Key Components:**
```rust
/// Extended subvolume info with all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsSubvolume {
    // Identity
    pub id: u64,
    pub path: PathBuf,
    pub parent_id: Option<u64>,
    
    // UUIDs - NEW
    pub uuid: Uuid,
    pub parent_uuid: Option<Uuid>,  // For snapshot tracking
    pub received_uuid: Option<Uuid>, // For send/receive
    
    // Timestamps - NEW
    pub created: DateTime<Local>,
    pub modified: DateTime<Local>,
    
    // Properties - NEW
    pub generation: u64,
    pub flags: u64,
    pub is_readonly: bool,
    pub is_default: bool,
    
    // Transaction IDs (for advanced users)
    pub ctransid: u64,
    pub otransid: u64,
}

impl From<btrfsutil::subvolume::SubvolumeInfo> for BtrfsSubvolume {
    // Conversion logic
}

/// BTRFS filesystem operations
pub struct BtrfsFilesystem {
    mount_point: PathBuf,
    helper: BtrfsHelper,  // Privilege escalation wrapper
}

impl BtrfsFilesystem {
    pub async fn new(mount_point: PathBuf) -> Result<Self>;
    
    // Basic operations (same API as before, enhanced return types)
    pub async fn list_subvolumes(&self) -> Result<Vec<BtrfsSubvolume>>;
    pub async fn create_subvolume(&self, name: &str) -> Result<BtrfsSubvolume>;
    pub async fn delete_subvolume(&self, path: &Path) -> Result<()>;
    pub async fn create_snapshot(&self, source: &Path, dest: &Path, readonly: bool) -> Result<BtrfsSubvolume>;
    
    // NEW: Advanced operations
    pub async fn get_subvolume_info(&self, path: &Path) -> Result<BtrfsSubvolume>;
    pub async fn set_readonly(&self, path: &Path, readonly: bool) -> Result<()>;
    pub async fn get_default_subvolume(&self) -> Result<BtrfsSubvolume>;
    pub async fn set_default_subvolume(&self, path: &Path) -> Result<()>;
    pub async fn list_deleted_subvolumes(&self) -> Result<Vec<BtrfsSubvolume>>;
    
    // NEW: Quota operations (basic support)
    pub async fn create_qgroup_inherit(&self, parent_qgroup: u64) -> Result<QgroupInherit>;
    
    // NEW: Validation
    pub async fn is_subvolume(&self, path: &Path) -> Result<bool>;
}

/// Privilege helper wrapper
struct BtrfsHelper {
    helper_path: PathBuf,
}

impl BtrfsHelper {
    fn new() -> Self;
    async fn execute(&self, operation: Operation) -> Result<Output>;
}

enum Operation {
    ListSubvolumes { mount_point: PathBuf },
    CreateSubvolume { mount_point: PathBuf, name: String },
    DeleteSubvolume { mount_point: PathBuf, path: PathBuf, recursive: bool },
    CreateSnapshot { mount_point: PathBuf, source: PathBuf, dest: PathBuf, readonly: bool, recursive: bool },
    SetReadonly { mount_point: PathBuf, path: PathBuf, readonly: bool },
    SetDefault { mount_point: PathBuf, path: PathBuf },
    GetDefault { mount_point: PathBuf },
    ListDeleted { mount_point: PathBuf },
}
```

**Implementation Notes:**
- All blocking btrfsutil calls wrapped in `tokio::spawn_blocking()`
- Privilege helper spawned via `tokio::process::Command`
- Error handling uses `anyhow` with context
- Serialization for helper IPC uses JSON or bincode

#### 1.3 Privilege Helper Binary
**New File:** `disks-btrfs-helper/src/main.rs` (new crate in workspace)

**Purpose:** Minimal privileged binary for CAP_SYS_ADMIN operations

```rust
use btrfsutil::subvolume::*;
use clap::{Parser, Subcommand};
use serde_json;

#[derive(Parser)]
#[command(name = "cosmic-ext-disks-btrfs-helper")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List { mount_point: PathBuf },
    Create { mount_point: PathBuf, name: String },
    Delete { mount_point: PathBuf, path: PathBuf, recursive: bool },
    Snapshot { mount_point: PathBuf, source: PathBuf, dest: PathBuf, readonly: bool },
    SetReadonly { mount_point: PathBuf, path: PathBuf, readonly: bool },
    SetDefault { mount_point: PathBuf, path: PathBuf },
    GetDefault { mount_point: PathBuf },
    ListDeleted { mount_point: PathBuf },
}

fn main() -> Result<()> {
    // Parse CLI
    let cli = Cli::parse();
    
    // Execute operation
    let result = match cli.command {
        Commands::List { mount_point } => list_subvolumes(&mount_point),
        Commands::Create { mount_point, name } => create_subvolume(&mount_point, &name),
        // ... other commands
    };
    
    // Output JSON result
    match result {
        Ok(output) => {
            println!("{}", serde_json::to_string(&output)?);
            Ok(())
        }
        Err(e) => {
            eprintln!("{:#}", e);
            std::process::exit(1);
        }
    }
}

fn list_subvolumes(mount_point: &Path) -> Result<Vec<SubvolumeOutputFormat>> {
    let root = Subvolume::try_from(mount_point)?;
    let iter = SubvolumeIterator::try_from(&root)?;
    
    let mut subvolumes = Vec::new();
    for subvol in iter {
        let subvol = subvol?;
        let info = subvol.info()?;
        subvolumes.push(SubvolumeOutputFormat::from(info));
    }
    
    Ok(subvolumes)
}

// ... other operation implementations

#[derive(Serialize, Deserialize)]
struct SubvolumeOutputFormat {
    // Serializable version of SubvolumeInfo
}
```

**Polkit Integration:**
**New File:** `data/com.system76.CosmicExtDisks.Btrfs.policy`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC
 "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <vendor>System76</vendor>
  <vendor_url>https://system76.com</vendor_url>

  <action id="com.system76.CosmicExtDisks.btrfs.manage">
    <description>Manage BTRFS subvolumes and snapshots</description>
    <message>Authentication is required to manage BTRFS filesystems</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin_keep</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/usr/libexec/cosmic-ext-disks-btrfs-helper</annotate>
  </action>
</policyconfig>
```

#### 1.4 Update Module Exports
**File:** `disks-dbus/src/disks/mod.rs`

```rust
// CHANGE:
mod btrfs;  // REMOVE
mod btrfs_native;  // ADD

// CHANGE:
pub use btrfs_native::{BtrfsFilesystem, BtrfsSubvolume};  // Updated exports
```

#### 1.5 Delete Old BTRFS Module
**File:** `disks-dbus/src/disks/btrfs.rs`

**Action:** DELETE ENTIRE FILE (289 lines)

**Rationale:** 
- Incompatible with new architecture
- Would cause confusion
- Clean break preferred

---

### Phase 2: State & Message Updates

#### 2.1 Enhanced BTRFS State
**File:** `disks-ui/src/ui/btrfs/state.rs`

```rust
// CHANGE: Add new fields
#[derive(Clone, Debug)]
pub struct BtrfsState {
    pub mount_point: Option<String>,
    pub block_path: Option<String>,
    
    // Existing
    pub subvolumes: Option<Result<Vec<BtrfsSubvolume>, String>>,
    pub loading: bool,
    pub expanded_subvolumes: HashMap<u64, bool>,
    pub used_space: Option<Result<u64, String>>,
    pub loading_usage: bool,
    
    // NEW: Enhanced metadata
    pub default_subvolume_id: Option<u64>,
    pub deleted_subvolumes: Option<Vec<BtrfsSubvolume>>,
    pub show_deleted: bool,  // Toggle for UI
    
    // NEW: Selected subvolume for properties
    pub selected_subvolume: Option<BtrfsSubvolume>,
    pub show_properties_dialog: bool,
}
```

#### 2.2 Expanded Messages
**File:** `disks-ui/src/ui/btrfs/message.rs`

```rust
// ADD new messages:
#[derive(Clone, Debug)]
pub enum Message {
    // Existing messages kept...
    
    // NEW: Metadata operations
    LoadDefaultSubvolume,
    DefaultSubvolumeLoaded(Result<BtrfsSubvolume, String>),
    SetDefaultSubvolume { subvolume_id: u64 },
    
    // NEW: Read-only control
    ToggleReadonly { subvolume_id: u64 },
    ReadonlyToggled(Result<(), String>),
    
    // NEW: Properties dialog
    ShowProperties { subvolume_id: u64 },
    CloseProperties,
    
    // NEW: Deleted subvolumes
    LoadDeletedSubvolumes,
    DeletedSubvolumesLoaded(Result<Vec<BtrfsSubvolume>, String>),
    ToggleShowDeleted,
    
    // NEW: Refresh
    RefreshAll,  // Reload everything
}
```

#### 2.3 Convert to New Types
**File:** `disks-ui/src/ui/app/message.rs`

```rust
// CHANGE: Update BTRFS message types
BtrfsLoadSubvolumes {
    block_path: String,
    mount_point: String,
},
BtrfsSubvolumesLoaded {
    mount_point: String,
    result: Result<Vec<disks_dbus::BtrfsSubvolume>, String>,  // Updated type
},

// ADD: New messages matching btrfs/message.rs additions
BtrfsToggleReadonly {
    mount_point: String,
    subvolume_id: u64,
},
BtrfsReadonlyToggled {
    mount_point: String,
    result: Result<(), String>,
},

BtrfsSetDefault {
    mount_point: String,
    subvolume_id: u64,
},
BtrfsDefaultSet {
    mount_point: String,
    result: Result<(), String>,
},

BtrfsShowProperties {
    mount_point: String,
    subvolume_id: u64,
},

// ... etc
```

---

### Phase 3: Update Handlers

#### 3.1 BTRFS Update Handler
**File:** `disks-ui/src/ui/app/update/btrfs.rs`

**Action:** COMPLETE REWRITE

**Key Changes:**
```rust
// REPLACE all async operations with new API

// Example: List subvolumes
pub fn load_subvolumes(
    mount_point: String,
    block_path: String,
) -> Task<cosmic::Action<Message>> {
    Task::future(async move {
        // NEW: Use btrfs_native instead of D-Bus
        let manager = Manager::new().await.ok()?;
        let btrfs = match disks_dbus::BtrfsFilesystem::new(
            PathBuf::from(&mount_point)
        ).await {
            Ok(fs) => fs,
            Err(e) => {
                return Some(cosmic::Action::App(Message::BtrfsSubvolumesLoaded {
                    mount_point,
                    result: Err(format!("Failed to initialize BTRFS: {}", e)),
                }));
            }
        };
        
        // List with full metadata
        match btrfs.list_subvolumes().await {
            Ok(subvolumes) => {
                Some(cosmic::Action::App(Message::BtrfsSubvolumesLoaded {
                    mount_point,
                    result: Ok(subvolumes),
                }))
            }
            Err(e) => {
                Some(cosmic::Action::App(Message::BtrfsSubvolumesLoaded {
                    mount_point,
                    result: Err(format!("Failed to list subvolumes: {}", e)),
                }))
            }
        }
    })
}

// ADD: New handlers for advanced operations

pub fn toggle_readonly(
    mount_point: String,
    subvolume_id: u64,
    current_state: bool,
) -> Task<cosmic::Action<Message>> {
    Task::future(async move {
        let btrfs = BtrfsFilesystem::new(PathBuf::from(&mount_point)).await.ok()?;
        
        // Find subvolume path from ID
        let subvolumes = btrfs.list_subvolumes().await.ok()?;
        let subvol = subvolumes.iter().find(|s| s.id == subvolume_id)?;
        
        // Toggle readonly flag
        match btrfs.set_readonly(&subvol.path, !current_state).await {
            Ok(()) => {
                Some(cosmic::Action::App(Message::BtrfsReadonlyToggled {
                    mount_point,
                    result: Ok(()),
                }))
            }
            Err(e) => {
                Some(cosmic::Action::App(Message::BtrfsReadonlyToggled {
                    mount_point,
                    result: Err(format!("Failed to toggle readonly: {}", e)),
                }))
            }
        }
    })
}

pub fn set_default_subvolume(
    mount_point: String,
    subvolume_id: u64,
) -> Task<cosmic::Action<Message>> {
    // Similar pattern...
}

pub fn load_deleted_subvolumes(
    mount_point: String,
) -> Task<cosmic::Action<Message>> {
    // Similar pattern...
}
```

---

### Phase 4: UI Enhancements

#### 4.1 Enhanced Subvolume Grid
**File:** `disks-ui/src/ui/btrfs/view.rs`

**Changes:**
```rust
// ADD: New columns to grid
// Current: Path | ID | Actions
// New: Path | ID | Created | Modified | Flags | Actions

fn render_subvolume_row<'a>(
    subvol: &'a BtrfsSubvolume,
    children_map: &HashMap<u64, Vec<&'a BtrfsSubvolume>>,
    state: &'a BtrfsState,
    indent_level: u16,
) -> Element<'a, Message> {
    // ... existing expander and path code ...
    
    // ADD: Created timestamp
    row_items.push(
        widget::text::caption(
            format_relative_time(&subvol.created)
        )
        .width(100)
        .into(),
    );
    
    // ADD: Modified timestamp
    row_items.push(
        widget::text::caption(
            format_relative_time(&subvol.modified)
        )
        .width(100)
        .into(),
    );
    
    // ADD: Flags/badges
    let mut badges = widget::row::with_capacity(3).spacing(4);
    
    if state.default_subvolume_id == Some(subvol.id) {
        badges = badges.push(
            widget::container(widget::text("DEFAULT").size(10))
                .padding(2)
                .style(cosmic::theme::Container::Custom(Box::new(|theme| {
                    container::Style {
                        background: Some(Background::Color(theme.cosmic().accent_color())),
                        text_color: Some(theme.cosmic().on_accent_color()),
                        border_radius: 4.0.into(),
                        ..Default::default()
                    }
                })))
        );
    }
    
    if subvol.is_readonly {
        badges = badges.push(
            widget::icon(widget::icon::from_name("changes-prevent-symbolic"))
                .size(12)
        );
    }
    
    if subvol.parent_uuid.is_some() {
        badges = badges.push(
            widget::icon(widget::icon::from_name("camera-photo-symbolic"))
                .size(12)
        );
    }
    
    row_items.push(badges.into());
    
    // MODIFY: Enhanced delete button → context menu
    let context_menu = widget::popover(
        widget::button::icon(widget::icon::from_name("view-more-symbolic")),
    )
    .popup({
        let mut menu = widget::column::with_capacity(5).padding(8).spacing(4);
        
        menu = menu.push(
            widget::button::text(fl!("btrfs-properties"))
                .on_press(Message::ShowProperties { subvolume_id: subvol.id })
                .width(Length::Fill)
        );
        
        menu = menu.push(widget::horizontal_rule(1));
        
        menu = menu.push(
            widget::button::text(if subvol.is_readonly {
                fl!("btrfs-make-writable")
            } else {
                fl!("btrfs-make-readonly")
            })
            .on_press(Message::ToggleReadonly { subvolume_id: subvol.id })
            .width(Length::Fill)
        );
        
        if !subvol.is_readonly {
            menu = menu.push(
                widget::button::text(fl!("btrfs-set-default"))
                    .on_press(Message::SetDefaultSubvolume { subvolume_id: subvol.id })
                    .width(Length::Fill)
            );
        }
        
        menu = menu.push(widget::horizontal_rule(1));
        
        menu = menu.push(
            widget::button::text(fl!("btrfs-delete"))
                .on_press(Message::BtrfsDeleteSubvolume {
                    block_path: state.block_path.clone().unwrap(),
                    mount_point: state.mount_point.clone().unwrap(),
                    path: subvol.path.to_string_lossy().to_string(),
                })
                .width(Length::Fill)
                .class(cosmic::theme::Button::Destructive)
        );
        
        menu.into()
    });
    
    row_items.push(context_menu.into());
    
    // ... rest of function ...
}

// ADD: Relative time formatting helper
fn format_relative_time(dt: &DateTime<Local>) -> String {
    let now = Local::now();
    let duration = now.signed_duration_since(*dt);
    
    if duration.num_seconds() < 60 {
        fl!("time-just-now")
    } else if duration.num_minutes() < 60 {
        fl!("time-minutes-ago", minutes = duration.num_minutes())
    } else if duration.num_hours() < 24 {
        fl!("time-hours-ago", hours = duration.num_hours())
    } else if duration.num_days() < 30 {
        fl!("time-days-ago", days = duration.num_days())
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

// ADD: Deleted subvolumes section
fn render_deleted_section<'a>(
    state: &'a BtrfsState,
) -> Element<'a, Message> {
    let header = widget::row::with_children(vec![
        widget::text(fl!("btrfs-deleted-subvolumes"))
            .size(14.0)
            .font(cosmic::iced::font::Font {
                weight: cosmic::iced::font::Weight::Semibold,
                ..Default::default()
            })
            .into(),
        widget::horizontal_space().into(),
        widget::button::icon(widget::icon::from_name(if state.show_deleted {
            "go-down-symbolic"
        } else {
            "go-next-symbolic"
        }))
        .on_press(Message::ToggleShowDeleted)
        .into(),
    ]);
    
    let mut content = widget::column::with_children(vec![header.into()]);
    
    if state.show_deleted {
        if let Some(Ok(deleted)) = &state.deleted_subvolumes {
            if deleted.is_empty() {
                content = content.push(
                    widget::text::caption(fl!("btrfs-no-deleted"))
                );
            } else {
                content = content.push(
                    widget::text::caption(
                        fl!("btrfs-deleted-count", count = deleted.len())
                    )
                );
                
                for subvol in deleted {
                    content = content.push(
                        widget::container(
                            widget::text::caption(format!("ID {}: {}", subvol.id, subvol.path.display()))
                        )
                        .padding([4, 8])
                    );
                }
                
                content = content.push(
                    widget::button::text(fl!("btrfs-cleanup-deleted"))
                        .on_press(Message::CleanupDeletedSubvolumes)
                );
            }
        } else if state.loading {
            content = content.push(widget::text::caption(fl!("loading")));
        }
    }
    
    content.spacing(8).into()
}
```

#### 4.2 Properties Dialog
**New File:** `disks-ui/src/ui/btrfs/properties.rs`

```rust
use cosmic::widget;
use cosmic::{Element, iced::Length};
use chrono::Local;
use uuid::Uuid;

use crate::fl;
use super::Message;
use disks_dbus::BtrfsSubvolume;

pub fn properties_dialog<'a>(
    subvol: &'a BtrfsSubvolume,
) -> Element<'a, Message> {
    let content = widget::column::with_children(vec![
        // Name/Path section
        property_row(fl!("name"), subvol.path.file_name().unwrap_or_default().to_string_lossy()),
        property_row(fl!("path"), subvol.path.to_string_lossy()),
        
        widget::horizontal_rule(1).into(),
        
        // Identity section
        property_row(fl!("subvolume-id"), subvol.id.to_string()),
        property_row(fl!("uuid"), subvol.uuid.to_string()),
        
        if let Some(parent_id) = subvol.parent_id {
            property_row(fl!("parent-id"), parent_id.to_string()).into()
        } else {
            widget::Space::new(0, 0).into()
        },
        
        widget::horizontal_rule(1).into(),
        
        // Timestamps section
        section_header(fl!("timestamps")),
        property_row(fl!("created"), subvol.created.format("%Y-%m-%d %H:%M:%S").to_string()),
        property_row(fl!("modified"), subvol.modified.format("%Y-%m-%d %H:%M:%S").to_string()),
        
        widget::horizontal_rule(1).into(),
        
        // Snapshot info (if applicable)
        if let Some(parent_uuid) = subvol.parent_uuid {
            widget::column::with_children(vec![
                section_header(fl!("snapshot-info")).into(),
                property_row(fl!("parent-uuid"), parent_uuid.to_string()).into(),
                widget::text::caption(fl!("snapshot-notice")).into(),
            ]).into()
        } else {
            widget::Space::new(0, 0).into()
        },
        
        widget::horizontal_rule(1).into(),
        
        // Properties section
        section_header(fl!("properties")),
        property_row(fl!("generation"), subvol.generation.to_string()),
        property_row(fl!("readonly"), if subvol.is_readonly { fl!("yes") } else { fl!("no") }),
        property_row(fl!("default"), if subvol.is_default { fl!("yes") } else { fl!("no") }),
        
        widget::horizontal_rule(1).into(),
        
        // Advanced section (collapsible)
        section_header(fl!("advanced")),
        property_row("ctransid", subvol.ctransid.to_string()),
        property_row("otransid", subvol.otransid.to_string()),
        property_row(fl!("flags"), format!("0x{:x}", subvol.flags)),
        
        if let Some(received_uuid) = subvol.received_uuid {
            property_row(fl!("received-uuid"), received_uuid.to_string()).into()
        } else {
            widget::Space::new(0, 0).into()
        },
    ])
    .spacing(8)
    .padding(16);
    
    widget::dialog(fl!("subvolume-properties"))
        .body(content)
        .primary_action(
            widget::button::standard(fl!("close"))
                .on_press(Message::CloseProperties)
        )
        .into()
}

fn section_header(text: String) -> Element<'static, Message> {
    widget::text(text)
        .size(13.0)
        .font(cosmic::iced::font::Font {
            weight: cosmic::iced::font::Weight::Semibold,
            ..Default::default()
        })
        .into()
}

fn property_row<'a>(
    label: impl Into<String>,
    value: impl Into<String>,
) -> Element<'a, Message> {
    widget::row::with_children(vec![
        widget::text(label.into())
            .width(Length::FillPortion(1))
            .into(),
        widget::text(value.into())
            .width(Length::FillPortion(2))
            .into(),
    ])
    .spacing(16)
    .into()
}
```

---

### Phase 5: Localization

#### 5.1 New Translation Strings
**File:** `disks-ui/i18n/en/cosmic_ext_disks.ftl`

```fluent
# BTRFS - Enhanced strings
btrfs-properties = Properties
btrfs-make-readonly = Make Read-Only
btrfs-make-writable = Make Writable
btrfs-set-default = Set as Default
btrfs-delete = Delete

# Timestamps
time-just-now = Just now
time-minutes-ago = {$minutes} {$minutes ->
    [one] minute
    *[other] minutes
} ago
time-hours-ago = {$hours} {$hours ->
    [one] hour
    *[other] hours
} ago
time-days-ago = {$days} {$days ->
    [one] day
    *[other] days
} ago

# Deleted subvolumes
btrfs-deleted-subvolumes = Deleted Subvolumes
btrfs-no-deleted = No deleted subvolumes pending cleanup
btrfs-deleted-count = {$count} {$count ->
    [one] subvolume
    *[other] subvolumes
} pending cleanup
btrfs-cleanup-deleted = Clean Up Now

# Properties dialog
subvolume-properties = Subvolume Properties
name = Name
path = Path
subvolume-id = Subvolume ID
uuid = UUID
parent-id = Parent ID
timestamps = Timestamps
created = Created
modified = Modified
snapshot-info = Snapshot Information
parent-uuid = Parent UUID
snapshot-notice = This subvolume is a snapshot of another subvolume
properties = Properties
generation = Generation
readonly = Read-Only
default = Default Boot Subvolume
yes = Yes
no = No
advanced = Advanced
received-uuid = Received UUID
flags = Flags

# Confirmations
confirm-set-default-title = Set Default Subvolume
confirm-set-default-body = Are you sure you want to set "{$name}" as the default boot subvolume? This will affect which subvolume is mounted at boot.
confirm-readonly-title = Make Read-Only
confirm-readonly-body = Making this subvolume read-only will prevent any modifications. This is recommended for snapshots you want to preserve.
confirm-writable-title = Make Writable
confirm-writable-body = Making this subvolume writable will allow modifications. Be careful not to accidentally modify important snapshots.
```

---

### Phase 6: Testing Infrastructure

#### 6.1 Integration Tests
**New File:** `disks-dbus/tests/btrfs_integration.rs`

```rust
use disks_dbus::BtrfsFilesystem;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
#[ignore] // Requires root and BTRFS support
async fn test_subvolume_lifecycle() {
    // Setup: Create loop device with BTRFS
    let temp_dir = TempDir::new().unwrap();
    let loop_device = setup_btrfs_loop_device(&temp_dir).await.unwrap();
    let mount_point = mount_btrfs(&loop_device).await.unwrap();
    
    // Test: Create filesystem handle
    let btrfs = BtrfsFilesystem::new(mount_point.path().to_path_buf())
        .await
        .unwrap();
    
    // Test: List initial subvolumes (should have root)
    let subvols = btrfs.list_subvolumes().await.unwrap();
    assert_eq!(subvols.len(), 1);
    assert_eq!(subvols[0].id, 5); // Root subvolume
    
    // Test: Create subvolume
    let created = btrfs.create_subvolume("test-subvol").await.unwrap();
    assert_eq!(created.path.file_name().unwrap(), "test-subvol");
    assert!(!created.is_readonly);
    
    // Test: List again (should have 2)
    let subvols = btrfs.list_subvolumes().await.unwrap();
    assert_eq!(subvols.len(), 2);
    
    // Test: Set readonly
    btrfs.set_readonly(&created.path, true).await.unwrap();
    let info = btrfs.get_subvolume_info(&created.path).await.unwrap();
    assert!(info.is_readonly);
    
    // Test: Create snapshot
    let snapshot = btrfs.create_snapshot(
        &created.path,
        &mount_point.path().join("snapshot-1"),
        true,
    ).await.unwrap();
    assert!(snapshot.is_readonly);
    assert_eq!(snapshot.parent_uuid, Some(created.uuid));
    
    // Test: Set default
    btrfs.set_default_subvolume(&created.path).await.unwrap();
    let default = btrfs.get_default_subvolume().await.unwrap();
    assert_eq!(default.id, created.id);
    
    // Test: Delete subvolume
    btrfs.delete_subvolume(&snapshot.path).await.unwrap();
    let deleted = btrfs.list_deleted_subvolumes().await.unwrap();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].id, snapshot.id);
    
    // Cleanup
    cleanup_btrfs(&loop_device, &mount_point).await.unwrap();
}

async fn setup_btrfs_loop_device(dir: &TempDir) -> Result<PathBuf> {
    // Create 1GB sparse file
    // Create loop device
    // Format as BTRFS
    // Return device path
}

async fn mount_btrfs(device: &Path) -> Result<TempDir> {
    // Create mount point
    // Mount via sudo
    // Return mount dir
}

async fn cleanup_btrfs(device: &Path, mount: &TempDir) -> Result<()> {
    // Unmount
    // Detach loop device
}
```

#### 6.2 Unit Tests
**File:** `disks-dbus/src/disks/btrfs_native.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_subvolume_conversion() {
        // Test SubvolumeInfo → BtrfsSubvolume conversion
    }
    
    #[test]
    fn test_operation_serialization() {
        // Test helper operation JSON serialization
    }
    
    #[tokio::test]
    async fn test_helper_invocation() {
        // Test helper binary can be spawned
    }
}
```

---

## Feature Additions (New Capabilities)

### 1. Snapshot Relationship Graph ⭐⭐⭐
**Priority:** High  
**Complexity:** Medium

**Feature:** Visual tree/graph showing snapshot relationships

**Implementation:**
- Use `parent_uuid` to build parent-child relationships
- Render as tree view with lines connecting snapshots to originals
- Click to navigate to parent/children
- Show snapshot count per subvolume

**UI Mockup:**
```
@root
├─ @home (3 snapshots)
│  ├─ @home-2026-02-01
│  ├─ @home-2026-02-10
│  └─ @home-2026-02-13
└─ @var
   └─ @var-backup
```

### 2. Quick Snapshot Shortcuts ⭐⭐⭐
**Priority:** High  
**Complexity:** Low

**Feature:** Right-click context menu with common operations

**Options:**
- "Quick Snapshot Now" - Creates timestamped snapshot
- "Revert to This Snapshot" - Restore from snapshot (advanced)
- "Compare with Parent" - Show differences (requires btrfs send/receive)
- "Schedule Snapshots" - Open scheduling dialog (future)

### 3. Subvolume Usage Breakdown ⭐⭐
**Priority:** Medium  
**Complexity:** Medium-High

**Feature:** Show per-subvolume disk usage (not just filesystem total)

**Challenge:** `libbtrfsutil` doesn't provide this, need to call `btrfs qgroup show`

**Implementation:**
- Fall back to CLI parsing for quota information
- Display "exclusive" and "referenced" space per subvolume
- Enable quota groups if not already enabled (with warning)
- Show pie chart of space usage by subvolume

### 4. Automatic Snapshot Naming ⭐⭐
**Priority:** Medium  
**Complexity:** Low

**Feature:** Smart defaults for snapshot names

**Patterns:**
- `{name}-{timestamp}` e.g., `@home-2026-02-13-1430`
- `{name}-before-{action}` e.g., `@root-before-update`
- Template system in preferences

**UI:**
- Dropdown with naming templates
- Preview of generated name
- Custom name option

### 5. Batch Snapshot Operations ⭐⭐
**Priority:** Medium  
**Complexity:** Medium

**Feature:** Select multiple subvolumes and operate on all

**Operations:**
- "Snapshot All" - Create snapshots of selected subvolumes
- "Delete All" - Batch delete with confirmation
- "Set All Read-Only" - Protect multiple snapshots

**UI:**
- Checkbox selection mode
- Batch action toolbar appears when items selected

### 6. Snapshot Scheduling (Timeshift-like) ⭐⭐⭐
**Priority:** High (but V3+)  
**Complexity:** High

**Feature:** Automatic periodic snapshots

**Requirements:**
- systemd timer/service integration
- Retention policy (keep last N snapshots)
- Pre/post hooks for package manager integration
- Email/notification on snapshot creation

**Out of scope for initial migration - separate feature**

### 7. Send/Receive Support ⭐⭐⭐
**Priority:** High (but V3+)  
**Complexity:** Very High

**Feature:** Replicate subvolumes to external drives/systems

**Operations:**
- `btrfs send` - Create sendstream of subvolume
- `btrfs receive` - Restore from sendstream
- Incremental sends (parent snapshot)
- Progress indication

**Out of scope for initial migration - requires CLI integration**

### 8. Compression Info per Subvolume ⭐
**Priority:** Low  
**Complexity:** Medium

**Feature:** Show compression algorithm and ratio

**Implementation:**
- Get from `btrfs filesystem show` or `compsize` tool
- Display compression type (zlib, lzo, zstd, none)
- Show space saved by compression
- Allow changing compression on subvolume

### 9. Subvolume Search/Filter ⭐
**Priority:** Low  
**Complexity:** Low

**Feature:** Search bar to filter subvolume list

**Filters:**
- By name/path
- By creation date range
- By type (snapshot vs regular)
- By readonly status
- By parent UUID (find all snapshots of X)

### 10. Export/Import Subvolume List ⭐
**Priority:** Low  
**Complexity:** Low

**Feature:** Save/load subvolume configurations

**Use Cases:**
- Document system state
- Recreate subvolume structure on new system
- Share configurations

**Format:** JSON or TOML file with subvolume metadata

---

## Build System Updates

### Cargo.toml Changes
**File:** `Cargo.toml` (workspace root)

```toml
[workspace]
members = [
    "disks-dbus",
    "disks-ui",
    "disks-btrfs-helper",  # NEW
]
```

**File:** `disks-btrfs-helper/Cargo.toml` (new)

```toml
[package]
name = "cosmic-ext-disks-btrfs-helper"
version = "0.1.0"
edition = "2021"

[dependencies]
btrfsutil = "0.2.0"
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"

[[bin]]
name = "cosmic-ext-disks-btrfs-helper"
path = "src/main.rs"
```

### Installation Updates
**File:** `justfile` or build scripts

```just
install-helper:
    # Build helper binary
    cargo build --release --package cosmic-ext-disks-btrfs-helper
    
    # Install to system libexec
    sudo install -Dm755 target/release/cosmic-ext-disks-btrfs-helper \
        /usr/libexec/cosmic-ext-disks-btrfs-helper
    
    # Install polkit policy
    sudo install -Dm644 data/com.system76.CosmicExtDisks.Btrfs.policy \
        /usr/share/polkit-1/actions/com.system76.CosmicExtDisks.Btrfs.policy

install: build install-helper
    # ... existing install steps
```

---

## Migration Timeline & Phases

### Week 1: Foundation
- ✅ Create investigation branch
- [ ] Add btrfsutil dependency
- [ ] Create btrfs_native.rs skeleton
- [ ] Implement basic list/create/delete operations
- [ ] Create helper binary skeleton
- [ ] Test basic operations with temporary BTRFS filesystem

**Deliverable:** Can list subvolumes with new API

### Week 2: Core Migration
- [ ] Complete all BtrfsFilesystem methods
- [ ] Implement helper binary operations
- [ ] Set up polkit integration
- [ ] Update state structures
- [ ] Update message types
- [ ] Rewrite update handlers

**Deliverable:** Feature parity with V1 (basic CRUD works)

### Week 3: UI Enhancements
- [ ] Enhanced subvolume grid (timestamps, badges)
- [ ] Properties dialog
- [ ] Context menus
- [ ] Read-only toggle UI
- [ ] Default subvolume badge
- [ ] Deleted subvolumes section

**Deliverable:** Rich UI with new features visible

### Week 4: Polish & Testing
- [ ] Write integration tests
- [ ] Write unit tests
- [ ] Add localization strings (all languages)
- [ ] Performance testing
- [ ] Memory leak testing  
- [ ] Error handling audit
- [ ] Documentation updates

**Deliverable:** Production-ready code

### Week 5-6: Advanced Features (Optional)
- [ ] Snapshot relationship graph
- [ ] Quick snapshot shortcuts
- [ ] Batch operations
- [ ] Search/filter
- [ ] Usage breakdown (if time permits)

**Deliverable:** V2.0 with advanced BTRFS support

---

## Risk Mitigation

### Risk 1: Helper Binary Security
**Issue:** Privileged binary is attack surface

**Mitigations:**
- Minimal code in helper (< 500 lines)
- No user input passed directly to system calls
- Path validation (must be under /mnt or /media)
- Audit logging of all operations
- Polkit authentication required
- Drop privileges after operation

### Risk 2: btrfsutil Crate Maintenance
**Issue:** Crate has low recent activity

**Mitigations:**
- Fork to cosmic-utils organization
- Maintain our own version if needed
- Crate is thin wrapper, easy to maintain
- Can vendor if absolutely necessary

### Risk 3: Regression from UDisks2
**Issue:** New implementation might have bugs

**Mitigations:**
- Extensive integration testing
- Beta testing period with opt-in flag
- Detailed error messages
- Rollback plan (keep V1 available)

### Risk 4: Privilege Escalation Complexity
**Issue:** More complex than UDisks2 model

**Mitigations:**
- Reuse existing polkit infrastructure
- Well-tested helper binary pattern
- Clear documentation
- Fallback to CLI if helper fails

---

## Success Criteria

### Must Have (Blocking V2.0 release)
- ✅ Feature parity with V1.0 BTRFS support
- ✅ All operations work as before
- ✅ No regressions in functionality
- ✅ Passing integration tests
- ✅ No memory leaks
- ✅ Helper binary security audit

### Should Have (V2.0 goals)
- ✅ Read-only toggle UI working
- ✅ Default subvolume management working
- ✅ Timestamp display working
- ✅ Properties dialog working
- ✅ Deleted subvolumes tracking working
- ✅ Context menus working

### Nice to Have (V2.1+)
- ⚠️ Snapshot relationship graph
- ⚠️ Batch operations
- ⚠️ Usage breakdown per subvolume
- ⚠️ Quick snapshot shortcuts

---

## Documentation Requirements

### User Documentation
- [ ] Update README with new features
- [ ] Create BTRFS management guide
- [ ] Screenshot updates showing new UI
- [ ] Troubleshooting guide for helper binary
- [ ] Migration guide from V1

### Developer Documentation
- [ ] Architecture documentation (this document)
- [ ] Helper binary protocol specification
- [ ] btrfsutil API usage patterns
- [ ] Testing guide for BTRFS features
- [ ] Contributing guide updates

---

## Post-Migration Cleanup

### Remove Obsolete Code
- [ ] Delete `disks-dbus/src/disks/btrfs.rs`
- [ ] Remove UDisks2 BTRFS references from docs
- [ ] Update dependency documentation
- [ ] Clean up unused imports

### Update Package Dependencies
- [ ] Update Arch PKGBUILD (remove udisks2-btrfs)
- [ ] Update Debian control file
- [ ] Update RPM spec file
- [ ] Update Flatpak manifest (if applicable)

---

## Recommended Feature Prioritization

### V2.0 Launch Features (Must Ship)
1. ✅ Full UDisks2 parity (list, create, delete, snapshot)
2. ✅ Read-only toggle
3. ✅ Default subvolume management
4. ✅ Timestamp display
5. ✅ Enhanced properties dialog
6. ✅ Deleted subvolumes tracking

### V2.1 Features (Nice to Have)
7. ⭐⭐⭐ Snapshot relationship visualization
8. ⭐⭐⭐ Quick snapshot context menu
9. ⭐⭐ Automatic snapshot naming templates
10. ⭐⭐ Batch operations UI

### V2.2+ Features (Future)
11. ⭐⭐⭐ Subvolume usage breakdown (requires quota)
12. ⭐⭐ Search and filter
13. ⭐ Compression info display
14. ⭐ Export/import configurations

### V3.0 Features (Major)
15. ⭐⭐⭐⭐ Snapshot scheduling (Timeshift-like)
16. ⭐⭐⭐⭐ Send/receive support

---

## Conclusion

This migration plan provides a **complete, destructive migration** from UDisks2 to btrfsutil with:

✅ **Clean architecture** - No legacy code, modern async patterns  
✅ **Security-first** - Minimal privileged helper binary with polkit  
✅ **Feature-rich** - 6 new major features enabled  
✅ **Well-tested** - Integration test suite with real BTRFS filesystems  
✅ **Documented** - Comprehensive docs for users and developers  
✅ **Maintainable** - Simpler codebase without dual implementations  

**Estimated Timeline:** 4-6 weeks for complete V2.0 release-ready migration

**Recommended Start:** Immediately after V1.0 release to allow ample testing time

**Next Step:** Create `refactor/btrfs-btrfsutil` branch and begin Week 1 tasks
