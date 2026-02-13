# BTRFS Feature Enhancements — Implementation Spec

**Branch:** `feature/btrfs-features`  
**Type:** Feature Addition (V2.1+)  
**Estimated Effort:** 7-8 weeks (10 features across 3 releases)  
**Status:** Planned

---

## Goal

Implement 10 high-value BTRFS management features unlocked by the btrfsutil migration to transform Cosmic Disks into a best-in-class subvolume manager.

**Unique Advantages:**
- Only GUI tool with snapshot relationship visualization
- Most user-friendly BTRFS subvolume manager  
- Integrated with COSMIC desktop environment
- Modern Rust/libcosmic implementation

---

## Scope

### In Scope
- ✅ **V2.1 Features** (Quick Wins + Major Features):
  1. Read-Only Protection Toggle ⭐⭐⭐⭐
  2. Creation Timestamps Display ⭐⭐⭐
  3. Automatic Snapshot Naming ⭐⭐⭐
  4. Snapshot Relationship Visualization ⭐⭐⭐⭐
  5. Default Subvolume Management ⭐⭐⭐
  6. Quick Snapshot Context Menu ⭐⭐⭐⭐
  
- ✅ **V2.2 Features** (Advanced Features):
  7. Batch Operations ⭐⭐⭐
  8. Subvolume Usage Breakdown ⭐⭐⭐
  9. Search & Filter ⭐⭐
  10. Deleted Subvolume Cleanup ⭐⭐

### Out of Scope
- ❌ Snapshot scheduling (V3.0)
- ❌ Send/receive support (V3.0)
- ❌ Automated cleanup policies (V3.0)
- ❌ Pre/post hooks for snapshots (V3.0)

---

## Feature Overview

### Quick Wins (V2.1 Priority - 2-3 weeks)

#### 1. Read-Only Protection Toggle ⭐⭐⭐⭐
**Value:** Prevents accidental snapshot modification  
**Complexity:** Low (0.5 weeks)

- Checkbox/toggle in subvolume list
- Lock icon indicator for read-only state
- Confirmation dialog when setting read-only
- Automatic read-only option for new snapshots

**Use Cases:**
- Protect critical backups from modification
- Ensure restore points remain pristine
- Regulatory compliance for immutable backups

#### 2. Creation Timestamps ⭐⭐⭐
**Value:** Essential for snapshot management  
**Complexity:** Very Low (0.3 weeks)

- Created/Modified columns in subvolume list
- Relative time display ("2 days ago")
- Tooltip with exact DateTime
- Sort by date functionality

**Use Cases:**
- Identify old snapshots for cleanup
- Correlate snapshots with system changes
- Implement retention policies

#### 3. Automatic Snapshot Naming ⭐⭐⭐
**Value:** Reduces cognitive load, improves organization  
**Complexity:** Low (0.5 weeks)

- Template system: `{name}`, `{date}`, `{time}`, `{action}`
- Smart defaults (Timestamped, Action-based, Sequential, Date-only)
- Live preview before creation
- Customizable templates in settings

**Templates:**
- Timestamped: `@home-2026-02-13-1430`
- Action-based: `@root-before-update`
- Sequential: `@home-snapshot-001`
- Date-only: `@var-2026-02-13`

#### 4. Default Subvolume Management ⭐⭐⭐
**Value:** Control boot configuration  
**Complexity:** Low-Medium (0.5 weeks)

- "DEFAULT" badge on default boot subvolume
- Button to set any subvolume as default
- Warning dialog about boot implications
- Highlight in different color

**Use Cases:**
- Rollback by setting older snapshot as default
- Test configurations without affecting current setup
- Multi-boot with different subvolume configurations

#### 5. Quick Snapshot Context Menu ⭐⭐⭐⭐
**Value:** Dramatically improves UX  
**Complexity:** Low (0.5 weeks)

- Right-click context menu on subvolumes
- Quick Snapshot Now with automatic naming
- Properties, Make Read-Only, Set Default, Delete
- Keyboard shortcuts (Ctrl+T for snapshot, Del for delete)

**Menu Items:**
- 📸 Quick Snapshot Now (Ctrl+T)
- Properties (Ctrl+I)
- 🔒 Make Read-Only / Make Writable
- 📌 Set as Default
- 🗑️ Delete (Del)

#### 6. Deleted Subvolume Cleanup ⭐⭐
**Value:** Reclaim space, housekeeping  
**Complexity:** Low (0.3 weeks)

- List deleted subvolumes pending cleanup
- Show space to be reclaimed
- One-click cleanup button
- Collapsible section in main view

**Technical:** Uses `Subvolume::deleted()` and `btrfs subvolume sync` command

---

### Major Features (V2.2 Priority - 2-3 weeks)

#### 7. Snapshot Relationship Visualization ⭐⭐⭐⭐
**Value:** Critical for understanding snapshot chains  
**Complexity:** Medium (1.5 weeks)

- Tree view showing parent-child relationships
- Visual lines connecting snapshots to originals
- Snapshot count badges on parent subvolumes
- Click to navigate to parent/children
- Highlight chain on hover

**Implementation:**
- Use `parent_uuid` to match against `uuid`
- Build graph: `HashMap<Uuid, Vec<BtrfsSubvolume>>`
- Cache relationships for performance
- Render using tree widget or custom drawing

**Display Options:**
- Tree view: Hierarchical with expand/collapse
- Relationship panel: Shows parent/children/siblings

#### 8. Batch Operations ⭐⭐⭐
**Value:** Saves time for bulk management  
**Complexity:** Medium (1.0 weeks)

- Checkbox selection mode
- Batch action toolbar when items selected
- Multi-select operations: snapshot, delete, set read-only
- Progress indicator for batch operations

**Operations:**
- Snapshot All - Create snapshots of selected subvolumes
- Delete All - Batch delete with single confirmation
- Set All Read-Only - Protect multiple snapshots
- Export List - Save selected subvolumes to file

#### 9. Subvolume Usage Breakdown ⭐⭐⭐
**Value:** Essential for capacity planning  
**Complexity:** High (2.0 weeks)

- Per-subvolume disk usage (not just filesystem total)
- Exclusive vs. Referenced space breakdown
- Pie chart / bar chart visualization
- Enable quota groups if needed (with warning)

**Technical Challenge:**
- btrfsutil doesn't provide quota info
- Must parse `btrfs qgroup show` output
- Requires quotas enabled (5-10% performance overhead)
- Show enable dialog with explanation

**Display:**
- Chart showing space distribution
- Table with Referenced/Exclusive/Shared columns
- Percentage of total for each subvolume

#### 10. Search & Filter ⭐⭐
**Value:** Quality of life for large filesystems  
**Complexity:** Low (0.5 weeks)

- Search bar to filter subvolume list
- Multiple filter criteria
- Real-time filtering as you type
- Saved filters for common queries

**Filter Types:**
- By Name/Path: Text matching
- By Date: Created before/after
- By Type: Regular subvolumes vs snapshots
- By Flag: Read-only, default, has children
- By Parent UUID: All snapshots of specific subvolume

---

## Acceptance Criteria

### V2.1 Launch (Quick Wins)
- [ ] **AC-1:** Read-only toggle functional with lock icon indicator
- [ ] **AC-2:** Timestamps displayed with relative time and tooltips
- [ ] **AC-3:** Automatic naming templates working with live preview
- [ ] **AC-4:** Default subvolume badge and set operation functional
- [ ] **AC-5:** Context menu with all operations and keyboard shortcuts
- [ ] **AC-6:** Deleted cleanup section shows pending subvolumes
- [ ] **AC-7:** All strings localized (English + Swedish optional)
- [ ] **AC-8:** No regressions in existing BTRFS functionality
- [ ] **AC-9:** All new features tested and documented

### V2.2 Launch (Advanced Features)
- [ ] **AC-10:** Snapshot relationship tree view functional
- [ ] **AC-11:** Batch selection and operations working
- [ ] **AC-12:** Usage breakdown showing with quota enable dialog
- [ ] **AC-13:** Search and filter working with all criteria
- [ ] **AC-14:** Performance acceptable for 200+ subvolumes
- [ ] **AC-15:** No memory leaks in long-running operations

### Quality Gates
- [ ] **AC-16:** `cargo test --workspace` passes
- [ ] **AC-17:** `cargo clippy --workspace` passes
- [ ] **AC-18:** `cargo fmt --check` passes
- [ ] **AC-19:** Manual testing checklist complete
- [ ] **AC-20:** Screenshots updated in README

---

## Implementation Priority Matrix

### V2.1 Features (Must Have - 2.6 weeks)
| Feature | Priority | Complexity | Weeks |
|---------|----------|------------|-------|
| Read-Only Toggle | ⭐⭐⭐⭐ | Low | 0.5 |
| Creation Timestamps | ⭐⭐⭐ | Very Low | 0.3 |
| Automatic Naming | ⭐⭐⭐ | Low | 0.5 |
| Default Subvolume | ⭐⭐⭐ | Low-Medium | 0.5 |
| Context Menu | ⭐⭐⭐⭐ | Low | 0.5 |
| Deleted Cleanup | ⭐⭐ | Low | 0.3 |

### V2.2 Features (High Value - 4.5 weeks)
| Feature | Priority | Complexity | Weeks |
|---------|----------|------------|-------|
| Snapshot Relationships | ⭐⭐⭐⭐ | Medium | 1.5 |
| Batch Operations | ⭐⭐⭐ | Medium | 1.0 |
| Usage Breakdown | ⭐⭐⭐ | High | 2.0 |
| Search & Filter | ⭐⭐ | Low | 0.5 |

**Total:** ~7-8 weeks for all 10 features

---

## Technical Architecture

### Data Structures

```rust
// Enhanced subvolume metadata
pub struct BtrfsSubvolume {
    // Already exists from btrfsutil migration
    pub id: u64,
    pub path: PathBuf,
    pub uuid: Uuid,
    pub parent_uuid: Option<Uuid>,
    pub created: DateTime<Local>,
    pub modified: DateTime<Local>,
    pub is_readonly: bool,
    pub is_default: bool,
    // ... other fields
}

// New: Snapshot relationship graph
pub struct SnapshotGraph {
    by_uuid: HashMap<Uuid, BtrfsSubvolume>,
    children: HashMap<Uuid, Vec<Uuid>>,
}

// New: Usage information
pub struct SubvolumeUsage {
    pub subvolume_id: u64,
    pub referenced: u64,  // bytes
    pub exclusive: u64,   // bytes
}

// New: Search/filter state
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

// New: Batch operation state
pub struct BatchProgress {
    pub operation: BatchOperation,
    pub total: usize,
    pub completed: usize,
    pub errors: Vec<(u64, String)>,
}
```

### UI State Extensions

```rust
pub struct BtrfsState {
    // Existing fields...
    pub subvolumes: Option<Result<Vec<BtrfsSubvolume>, String>>,
    pub expanded_subvolumes: HashMap<u64, bool>,
    
    // New: Usage tracking
    pub usage_data: Option<Result<Vec<SubvolumeUsage>, String>>,
    pub quotas_enabled: Option<bool>,
    
    // New: Filtering
    pub filter: SubvolumeFilter,
    pub filter_dialog_open: bool,
    
    // New: Batch operations
    pub selection_mode: bool,
    pub selected_subvolumes: HashSet<u64>,
    pub batch_progress: Option<BatchProgress>,
    
    // New: Relationship visualization
    pub snapshot_graph: Option<SnapshotGraph>,
    pub show_tree_view: bool,
    
    // New: Deleted subvolumes
    pub deleted_subvolumes: Option<Result<Vec<BtrfsSubvolume>, String>>,
    pub deleted_section_expanded: bool,
}
```

---

## User Persona Alignment

### Persona 1: Casual User
**Needs:** Easy backups before system changes  
**Priority Features:**
- ⭐⭐⭐⭐ Quick snapshot context menu
- ⭐⭐⭐ Automatic naming
- ⭐⭐⭐ Read-only protection

### Persona 2: Power User
**Needs:** Advanced snapshot management, scripting support  
**Priority Features:**
- ⭐⭐⭐⭐ Snapshot relationships
- ⭐⭐⭐ Batch operations
- ⭐⭐⭐ Usage breakdown
- ⭐⭐ Search & filter

### Persona 3: System Administrator
**Needs:** Multi-system management, compliance  
**Priority Features:**
- ⭐⭐⭐ Default subvolume (boot control)
- ⭐⭐⭐ Usage breakdown
- ⭐⭐⭐ Batch operations
- ⭐⭐ Deleted cleanup

---

## Competitive Analysis

| Feature | Cosmic Disks V2.1+ | GNOME Disks | Timeshift | Snapper | btrfs CLI |
|---------|-------------------|-------------|-----------|---------|-----------|
| GUI | ✅ Modern | ✅ Basic | ✅ Basic | ❌ | ❌ |
| Snapshot Relationships | ✅ Visual Tree | ❌ | ❌ | ❌ | ❌ |
| Batch Operations | ✅ | ❌ | ❌ | ❌ | ✅ |
| Usage Breakdown | ✅ | ❌ | ❌ | ❌ | ✅ |
| Context Menu | ✅ | ❌ | ❌ | ❌ | ❌ |
| Automatic Naming | ✅ Templates | ❌ | ✅ | ✅ | ❌ |
| Scheduling | V3.0 | ❌ | ✅ | ✅ | ❌ |
| Send/Receive | V3.0 | ❌ | ❌ | ❌ | ✅ |

**Our Niche:** Best-in-class GUI for BTRFS subvolume management with unique visualization features

---

## Dependencies

### No New Crates Required
All features use existing dependencies from btrfsutil migration:
- `btrfsutil = "0.2.0"` (already added)
- `uuid = "1.10"` (already added)
- `chrono = "0.4"` (already added)  

### System Dependencies
- `btrfs-progs` (already required for BTRFS support)
- For usage breakdown: quotas must be enabled (user action)

---

## Risk Mitigation

### Risk 1: Feature Creep
**Impact:** Development time exceeds 8 weeks  
**Mitigation:**
- Strict prioritization by star rating
- Ship V2.1 with 6 features, V2.2 with 4 features
- Can defer V2.2 features to V2.3 if needed

### Risk 2: Performance with Large Filesystems
**Impact:** UI sluggish with 200+ subvolumes  
**Mitigation:**
- Virtual scrolling for subvolume list
- Lazy loading of relationship graph
- Background tasks for usage calculations
- Caching with invalidation

### Risk 3: Quota Overhead
**Impact:** Users disable quotas due to performance  
**Mitigation:**
- Clear explanation of overhead (5-10%)
- Make quotas optional (disable usage breakdown if not enabled)
- Provide opt-in dialog with pros/cons

### Risk 4: UI Complexity
**Impact:** Interface becomes cluttered  
**Mitigation:**
- Progressive disclosure (context menus, expandable sections)
- Sane defaults (hide advanced features initially)
- User testing with casual users

---

## Success Metrics

### V2.1 Launch
- ✅ All 6 quick win features functional
- ✅ User surveys show "ease of use" improvement
- ✅ No critical bugs in first 2 weeks
- ✅ Performance acceptable for 50-100 subvolumes

### V2.2 Launch
- ✅ All 10 features complete
- ✅ Performance acceptable for 200+ subvolumes
- ✅ Competitive feature parity with Timeshift/Snapper
- ✅ User testimonials highlight unique advantages

---

## References

- **Feature Source:** `.copi/specs/investigate/btrfs-btrfsutil/feature-recommendations.md`
- **Foundation:** `.copi/specs/investigate/btrfs-btrfsutil/` (btrfsutil migration)
- **btrfsutil Docs:** https://docs.rs/btrfsutil
- **BTRFS Wiki:** https://btrfs.wiki.kernel.org/

---

## Implementation Notes

This is an **additive feature set** building on the btrfsutil migration. No breaking changes, only enhancements.

See `tasks.md` for detailed implementation steps organized by release (V2.1, V2.2).
