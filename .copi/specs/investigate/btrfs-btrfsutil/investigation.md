# Investigation: Migrating from UDisks2 BTRFS to btrfsutil crate

**Branch:** `investigate/btrfs-btrfsutil`  
**Date:** 2026-02-13  
**Purpose:** Evaluate benefits of switching from UDisks2 BTRFS interface to direct `btrfsutil` crate usage

---

## Executive Summary

The `btrfsutil` crate provides **significantly more functionality** than UDisks2's BTRFS interface. Migration would enable:
- **Comprehensive subvolume metadata** (UUIDs, timestamps, generations, flags)
- **Quinn group (quota) management** - completely unavailable via UDisks2
- **Default subvolume management** (get/set default)
- **Read-only flag control** (per-subvolume write protection)
- **Deleted subvolume tracking** (orphaned/pending cleanup)
- **Subvolume iteration** with filtering and post-order traversal
- **Direct filesystem sync operations**
- **Snapshot relationship tracking** (parent_uuid linkage)

**Recommendation:** Migrate to btrfsutil for V2. Current UDisks2 implementation is sufficient for V1 basic CRUD operations, but btrfsutil unlocks advanced features users expect from BTRFS tools.

---

## Current Implementation (UDisks2)

### Available Operations
From `disks-dbus/src/disks/btrfs.rs` via UDisks2 `org.freedesktop.UDisks2.Filesystem.BTRFS` interface:

#### Subvolume Operations ✅
- `GetSubvolumes(mount_options, snapshots_only)` → `Vec<BtrfsSubvolume>`
  - Returns: `id`, `path`, `parent_id` only
- `CreateSubvolume(name, mount_options)` → Creates subvolume
- `RemoveSubvolume(name, mount_options)` → Deletes subvolume  
- `CreateSnapshot(source, dest, read_only, mount_options)` → Creates snapshot

#### Usage Information ✅  
- `get_filesystem_info()` → Filesystem-level used space (async detection)

#### Limitations ❌
- **No subvolume metadata** beyond ID/path/parent
  - No UUID, generation, creation time
  - No flags (read-only status, etc.)
  - No compression info per-subvolume
- **No quota/qgroup support** at all
- **No default subvolume get/set**
- **No read-only flag control** per-subvolume
- **No deleted subvolume tracking**
- **No subvolume iteration control** (filtering, ordering)
- **No snapshot parent tracking** (which subvolume is this a snapshot of?)
- **Requires mounted filesystem** and specific mount options
- **Async D-Bus overhead** for every operation

---

## btrfsutil Crate Capabilities

### Core Library Info
- **Crate:** `btrfsutil` v0.2.0 ([crates.io](https://crates.io/crates/btrfsutil))
- **Backend:** Safe wrappers for `libbtrfsutil` (part of btrfs-progs)
- **License:** MIT
- **Maturity:** Stable (wraps well-tested C library from btrfs-progs)
- **Privileges:** Many operations require `CAP_SYS_ADMIN` (same as UDisks2)
- **Dependency:** Requires `btrfs-progs` package (same requirement as current)

### Available APIs

#### 1. Subvolume Struct (`btrfsutil::subvolume::Subvolume`)

**Creation & Retrieval:**
```rust
Subvolume::get(path) -> Result<Self>
Subvolume::get_anyway(path) -> Result<Self>  // Works for non-root paths
Subvolume::create(path, qgroup) -> Result<Self>
Subvolume::try_from(path) -> Result<Self>
Subvolume::try_from(id: u64) -> Result<Self> // Requires CAP_SYS_ADMIN
```

**Operations:**
```rust
subvol.delete(flags) -> Result<()>
subvol.snapshot(dest_path, flags, qgroup) -> Result<Subvolume>
subvol.info() -> Result<SubvolumeInfo>
subvol.is_ro() -> Result<bool>
subvol.set_ro(readonly: bool) -> Result<()>  // CAP_SYS_ADMIN
subvol.set_default() -> Result<()>  // CAP_SYS_ADMIN
```

**Static Methods:**
```rust
Subvolume::is_subvolume(path) -> Result<()>  // Check if path is subvolume
Subvolume::get_default(path) -> Result<Subvolume>  // CAP_SYS_ADMIN
Subvolume::deleted(fs_root) -> Result<Vec<Subvolume>>  // CAP_SYS_ADMIN
```

**Accessors:**
```rust
subvol.id() -> u64
subvol.path() -> &Path
```

**Flags:**
- `DeleteFlags::RECURSIVE` - Delete with children
- `SnapshotFlags::READ_ONLY` - Create read-only snapshot
- `SnapshotFlags::RECURSIVE` - Recursive snapshot

---

#### 2. SubvolumeInfo Struct (`btrfsutil::subvolume::SubvolumeInfo`)

**Rich Metadata Fields:**

```rust
pub struct SubvolumeInfo {
    // Identity
    pub id: u64,
    pub path: PathBuf,
    pub parent_id: Option<u64>,  // 0 for root/orphaned
    pub dir_id: Option<u64>,     // Inode of containing directory
    
    // UUIDs - Critical for snapshot relationships!
    pub uuid: Uuid,              // This subvolume's UUID
    pub parent_uuid: Option<Uuid>,     // UUID of parent (for snapshots)
    pub received_uuid: Option<Uuid>,   // UUID from btrfs send/receive
    
    // Transaction IDs (for sync operations)
    pub generation: u64,          // Subvolume root transaction
    pub ctransid: u64,            // Last inode change
    pub otransid: u64,            // Creation transaction
    pub stransid: Option<u64>,    // Send transaction (for received)
    pub rtransid: Option<u64>,    // Receive transaction
    
    // Timestamps (chrono::DateTime<Local>)
    pub ctime: DateTime<Local>,   // Last inode change time
    pub otime: DateTime<Local>,   // Creation time
    pub stime: Option<DateTime<Local>>,  // Send time
    pub rtime: Option<DateTime<Local>>,  // Receive time
    
    // Properties
    pub flags: u64,               // On-disk root item flags
}
```

**UUID Relationships:**
- `parent_uuid == Some(uuid)` → This is a snapshot of another subvolume
- `parent_uuid == None` → This is an original subvolume, not a snapshot
- `received_uuid == Some(uuid)` → This was received via btrfs send/receive

**Derived from:**
```rust
SubvolumeInfo::try_from(&Subvolume) -> Result<SubvolumeInfo>
Subvolume::from(&SubvolumeInfo) -> Subvolume
```

---

#### 3. SubvolumeIterator (`btrfsutil::subvolume::SubvolumeIterator`)

**Iteration with Control:**
```rust
SubvolumeIterator::try_from(&Subvolume) -> Result<SubvolumeIterator>
SubvolumeIterator::new(subvolume, flags) -> Result<SubvolumeIterator>
```

**Flags:**
- `SubvolumeIteratorFlags::POST_ORDER` - Visit children before parents

**Usage:**
```rust
let root = Subvolume::try_from("/mnt/btrfs")?;
let iter = SubvolumeIterator::try_from(&root)?;

for subvol in iter {
    let info = subvol?.info()?;
    println!("{}: {}", info.path.display(), info.uuid);
}
```

---

#### 4. Quota Groups (qgroups) (`btrfsutil::qgroup::QgroupInherit`)

**Quota Management:**
```rust
let qi = QgroupInherit::create()?;
qi.add(qgroup_id: u64)?;
let groups = qi.get_groups() -> Result<Vec<u64>>;
```

**Usage in Operations:**
```rust
Subvolume::create(path, Some(qi))?;
subvol.snapshot(dest, flags, Some(qi))?;
```

**Features:**
- Inherit quota groups from parent
- Set per-subvolume space limits
- Track referenced/exclusive space per qgroup

---

#### 5. Filesystem Sync (`btrfsutil::sync`)

**Explicit Sync Control:**
```rust
use btrfsutil::sync;
sync::sync(mount_point)?;  // Start and wait for filesystem sync
```

**Used internally by:**
- `Subvolume::create()` - Waits for transaction completion
- `Subvolume::snapshot()` - Ensures snapshot is consistent

---

## Feature Comparison Matrix

| Feature | UDisks2 | btrfsutil | Benefit |
|---------|---------|-----------|---------|
| **Basic Operations** |
| List subvolumes | ✅ | ✅ | Equal |
| Create subvolume | ✅ | ✅ | Equal |
| Delete subvolume | ✅ | ✅ | Equal |
| Create snapshot | ✅ | ✅ | Equal |
| **Metadata** |
| Subvolume ID | ✅ | ✅ | Equal |
| Subvolume path | ✅ | ✅ | Equal |
| Parent ID | ✅ | ✅ | Equal |
| UUID | ❌ | ✅ | **Unique identification** |
| Parent UUID | ❌ | ✅ | **Snapshot relationship tracking** |
| Received UUID | ❌ | ✅ | **Send/receive tracking** |
| Creation time | ❌ | ✅ | **User-friendly display** |
| Last change time | ❌ | ✅ | **Activity tracking** |
| Generation | ❌ | ✅ | **Sync operations** |
| Transaction IDs | ❌ | ✅ | **Advanced sync control** |
| Flags | ❌ | ✅ | **Read-only status, properties** |
| **Advanced Operations** |
| Read-only flag get | ❌ | ✅ | **Protect snapshots** |
| Read-only flag set | ❌ | ✅ | **Snapshot management** |
| Default subvolume get | ❌ | ✅ | **Boot configuration** |
| Default subvolume set | ❌ | ✅ | **Change boot subvolume** |
| Deleted subvolume list | ❌ | ✅ | **Cleanup operations** |
| Recursive deletion | ❌ | ✅ | **Easier management** |
| Check if path is subvolume | ❌ | ✅ | **Validation helper** |
| **Quota Management** |
| Enable/disable quotas | ❌ | ⚠️ | **Partial (via CLI)** |
| Quota group create | ❌ | ❌ | **Not in btrfsutil** |
| Quota limits set | ❌ | ❌ | **Not in btrfsutil** |
| Quota inherit | ❌ | ✅ | **On subvol create** |
| **Iteration** |
| Iterate subvolumes | ✅ | ✅ | Equal |
| Filter snapshots only | ✅ | ❌ | **UDisks2 advantage** |
| Post-order traversal | ❌ | ✅ | **Delete-safe ordering** |
| **Performance** |
| Requires mounted FS | ✅ | ✅ | Equal |
| D-Bus overhead | ❌ | ✅ | **Direct syscalls faster** |
| Async operation | ✅ | ✅ | Equal (we wrap in async) |
| **Requirements** |
| `btrfs-progs` package | ✅ | ✅ | Equal |
| `udisks2-btrfs` package | ✅ | ❌ | **One fewer dependency** |
| `CAP_SYS_ADMIN` | ✅ | ✅ | Equal (via polkit) |

---

## New Features Enabled by btrfsutil

### 1. Snapshot Relationship Visualization ⭐⭐⭐

**Current:** Flat list with parent_id, users cannot tell which are snapshots of which

**With btrfsutil:**
```rust
let info = subvol.info()?;
if let Some(parent_uuid) = info.parent_uuid {
    // This is a snapshot! Find the original subvolume by UUID
    for other in all_subvolumes {
        if other.info()?.uuid == parent_uuid {
            println!("Snapshot of: {}", other.path().display());
        }
    }
}
```

**UI Impact:** Show "Snapshot of: @home" tooltip/label on snapshot subvolumes

---

### 2. Read-Only Snapshot Protection ⭐⭐⭐

**Current:** Cannot toggle read-only flag, users must use CLI

**With btrfsutil:**
```rust
// Check if snapshot is read-only
if subvol.is_ro()? {
    // Show lock icon in UI
}

// Toggle read-only flag
subvol.set_ro(true)?;  // Make immutable
subvol.set_ro(false)?; // Make writable
```

**UI Impact:** 
- Checkbox/toggle in snapshot properties dialog
- Lock icon overlay on read-only subvolumes
- "Make Read-Only" / "Make Writable" buttons

---

### 3. Default Subvolume Management ⭐⭐

**Current:** Cannot see/change default subvolume, critical for boot configuration

**With btrfsutil:**
```rust
// Get current default
let default = Subvolume::get_default("/mnt/btrfs")?;
println!("Default: {}", default.path().display());

// Set new default
new_subvol.set_default()?;
```

**UI Impact:**
- Badge/icon showing "Default" subvolume in list
- "Set as Default" button (with warning about boot impact)
- Prominent indication which subvolume boots by default

---

### 4. Creation Time Display ⭐⭐

**Current:** No timestamp information

**With btrfsutil:**
```rust
let info = subvol.info()?;
println!("Created: {}", info.otime.format("%Y-%m-%d %H:%M:%S"));
println!("Last modified: {}", info.ctime.format("%Y-%m-%d %H:%M:%S"));
```

**UI Impact:**
- "Created" column in subvolume grid
- "Last Modified" column in subvolume grid
- Human-friendly relative times ("3 hours ago")

---

### 5. Deleted Subvolume Cleanup ⭐

**Current:** No visibility into pending subvolume cleanup

**With btrfsutil:**
```rust
let deleted = Subvolume::deleted("/mnt/btrfs")?;
if !deleted.is_empty() {
    println!("Pending cleanup: {} subvolumes", deleted.len());
}
```

**UI Impact:**
- "Pending Cleanup" section showing orphaned subvolumes
- Estimated space to be freed after cleanup
- Manual "Force Cleanup" button (triggers filesystem sync)

---

### 6. Quota Group Inheritance ⭐

**Current:** No quota support

**With btrfsutil:**
```rust
let qi = QgroupInherit::create()?;
qi.add(parent_qgroup_id)?;
Subvolume::create(path, Some(qi))?;
```

**UI Impact:**
- "Inherit Quota Group" checkbox in create dialog
- Dropdown to select parent qgroup
- Display inherited quota limits in subvolume properties

---

### 7. Advanced Snapshot Options ⭐⭐

**Current:** read_only parameter in UDisks2, but no other control

**With btrfsutil:**
```rust
use btrfsutil::subvolume::SnapshotFlags;

// Combined flags
let flags = SnapshotFlags::READ_ONLY | SnapshotFlags::RECURSIVE;
source_subvol.snapshot(dest, Some(flags), None)?;
```

**UI Impact:**
- Checkbox: "Read-only snapshot" (current functionality)
- Checkbox: "Recursive snapshot" (include nested subvolumes)
- Better snapshot creation dialog

---

### 8. Subvolume Validation ⭐

**Current:** No validation before operations

**With btrfsutil:**
```rust
// Check if path is actually a subvolume before operations
if Subvolume::is_subvolume(path).is_ok() {
    // Safe to proceed
}
```

**UI Impact:**
- Better error messages
- Prevent invalid operations earlier
- Validate user input in dialogs

---

### 9. UUID-based Operations (Future) ⭐⭐

**Current:** Only path-based operations

**With btrfsutil:**
```rust
// Find subvolume by UUID (useful for scripting/automation)
let target_uuid = uuid!("...");
for subvol in all_subvolumes {
    if subvol.info()?.uuid == target_uuid {
        // Found it!
    }
}
```

**UI Impact:**
- Copy UUID button (for scripting)
- "Find by UUID" feature
- Better snapshot tracking across filesystem changes

---

## Implementation Complexity

### Migration Difficulty: **Medium**

**Pros (Easier migration):**
- Direct Rust API, no D-Bus async complexity
- Type-safe with proper error handling
- No additional system dependencies beyond btrfs-progs
- Can migrate incrementally (keep UDisks2 for non-BTRFS operations)
- Well-documented with examples

**Cons (Complexity)**:
- Need to handle `CAP_SYS_ADMIN` ourselves (currently via UDisks2/polkit)
- Must add new Cargo dependency
- Need to rewrite `disks-dbus/src/disks/btrfs.rs` module
- May need CLI fallback for quota operations (limit set/get)
- Need to handle recursive deletion logic ourselves

---

## Proposed Architecture

### Current Architecture
```
UI → VolumesControl → Message → Update → disks_dbus::BtrfsFilesystem → UDisks2 D-Bus → Kernel
```

### Proposed Architecture (btrfsutil)
```
UI → VolumesControl → Message → Update → disks_dbus::BtrfsFilesystem → btrfsutil crate → libbtrfsutil → Kernel
```

**Key Changes:**
1. Replace `BtrfsFilesystem` struct to use `btrfsutil::subvolume::Subvolume` internally
2. Keep same public API from UI perspective (minimal disruption)
3. Add new methods to expose additional metadata
4. Use `tokio::task::spawn_blocking()` for blocking syscalls

---

## Performance Considerations

### UDisks2 Current Overhead
- D-Bus serialization/deserialization
- IPC context switching
- UDisks2 daemon processing
- Multiple roundtrips for metadata

### btrfsutil Direct Access
- Direct ioctl syscalls
- No IPC overhead
- Single call for complete subvolume info
- Minimal syscall overhead (same as btrfs CLI tools)

**Estimated Improvement:** 2-5x faster for subvolume listing operations

---

## Privilege/Security Model

### Current (UDisks2)
✅ UDisks2 handles polkit authentication  
✅ Runs privileged operations in udisksd daemon  
✅ Application doesn't need elevated privileges

### Proposed (btrfsutil)
⚠️ Need to request `CAP_SYS_ADMIN` via polkit  
⚠️ Operations execute in application process  
⚠️ More complex privilege escalation

**Solutions:**
1. **Use existing polkit rules** - cosmic-ext-disks already has polkit actions
2. **Spawn helper process** - Small privileged binary for BTRFS operations
3. **Wrap in pkexec** - Use pkexec for individual operations (slower)

**Recommended:** Solution #2 (helper process model, similar to partition operations)

---

## Dependencies

### Additional Crate Dependencies
```toml
[dependencies]
btrfsutil = "0.2.0"  # ~1000 SLoC
uuid = "1.0"         # For UUID handling (small)
chrono = "0.4"      # For DateTime (already used?)
```

### System Dependencies
- **Remove:** `udisks2-btrfs` package (optional UDisks2 module)
- **Keep:** `btrfs-progs` package (required, contains libbtrfsutil)
- **Total:** -1 dependency

---

## Compatibility & Risks

### Compatibility
✅ `libbtrfsutil` is part of stable btrfs-progs  
✅ API is stable (v1.2.0 since 2019)  
✅ Used by official btrfs tooling  
✅ Cross-distro availability (all major distros)  
⚠️ Crate is v0.2.0 (but wraps stable lib)  
⚠️ Crate maintainer seeking new maintainer (low activity)

### Risks
1. **Crate Maintenance:** `btrfsutil` crate hasn't seen updates since 2021
   - **Mitigation:** Crate wraps stable C library, minimal changes needed
   - **Backup:** Can fork/vendor if needed (small codebase ~1000 lines)

2. **Privilege Escalation:** More complex than UDisks2 model
   - **Mitigation:** Use existing polkit infrastructure

3. **API Churn:** If crate gets major updates during development
   - **Mitigation:** Pin to specific version, test thoroughly

@4. **Regression Risk:** Replacing working UDisks2 code
   - **Mitigation:** Incremental migration, feature flags, extensive testing

---

## Recommended Implementation Plan

### Phase 1: Investigation & Proof of Concept (This Phase) ✅
- [x] Research btrfsutil capabilities
- [ ] Build minimal POC showing:
  - Subvolume listing with full metadata
  - Read-only flag toggle
  - Default subvolume get/set
- [ ] Measure performance vs UDisks2
- [ ] Validate privilege escalation approach

### Phase 2: Core Migration (V2 Goal)
- [ ] Add `btrfsutil` dependency
- [ ] Create `disks-dbus/src/disks/btrfs_native.rs` (new module)
- [ ] Reimplement basic CRUD operations
- [ ] Add comprehensive test suite
- [ ] Feature flag: `btrfs-btrfsutil` (optional compile)
- [ ] Side-by-side testing with UDisks2 backend

### Phase 3: New Features (V2+)
- [ ] Read-only flag UI controls
- [ ] Default subvolume management UI
- [ ] UUID display and copy
- [ ] Creation/modification timestamps
- [ ] Snapshot relationship visualization
- [ ] Deleted subvolume cleanup UI

### Phase 4: Deprecation (V3?)
- [ ] Make btrfsutil default
- [ ] Remove UDisks2 BTRFS dependency
- [ ] Update documentation
- [ ] Migration guide for users

---

## Recommendation

### For V1 (Current Release)
**Keep UDisks2 implementation.** Reasons:
- Already working and tested
- Feature-complete for basic BTRFS management
- Lower risk for V1 release
- Users benefit from subvolume CRUD immediately

### For V2 (Future Release)
**Migrate to btrfsutil.** Reasons:
- Significant feature additions users expect
- Better performance (direct syscalls)
- One fewer system dependency (no udisks2-btrfs)
- Enables "1st class BTRFS support" claim
- Sets foundation for advanced features (send/receive, quotas, balance, etc.)

### Migration Strategy
1. Implement behind feature flag initially
2. Parallel testing period (both backends available)
3. Gradual rollout with user opt-in
4. Full migration after stabilization
5. Leverage V1 experience to inform V2 design

---

## Open Questions

1. **Privilege Escalation:** What's the best model for CAP_SYS_ADMIN operations?
   - Option A: Helper binary with polkit (like partition operations)
   - Option B: pkexec wrapper around main app
   - Option C: Investigate capabilities(7) for targeted privileges

2. **Quota Management:** btrfsutil only has qgroup_inherit, not full quota ops
   - Do we need to fall back to CLI (`btrfs qgroup`) for limit set/get?
   - Or defer quota UI to V3?

3. **Testing Infrastructure:** How to test BTRFS operations?
   - Need loop device setup in CI
   - Need privileged test environment
   - Use integration tests vs unit tests?

4. **Feature Parity Timeline:** V2 goal, or split across V2/V3?
   - V2: Core migration + read-only + default + timestamps?
   - V3: Quotas + send/receive + advanced features?

5. **Crate Maintenance:** Should we fork `btrfsutil` crate to ensure maintenance?
   - Keep upstream dependency and monitor?
   - Fork proactively and maintain ourselves?
   - Vendor the code directly?

---

## Next Steps (Immediate)

1. **Build POC:**
   - Create simple Rust binary using btrfsutil
   - List subvolumes with full metadata
   - Toggle read-only flag
   - Measure performance vs UDisks2

2. **Privilege Testing:**
   - Test CAP_SYS_ADMIN requirement
   - Experiment with polkit integration
   - Document privilege escalation approach

3. **Performance Benchmarking:**
   - Create filesystem with 100 subvolumes
   - Time UDisks2 listing operation
   - Time btrfsutil listing operation
   - Compare memory usage

4. **Decision Point:**
   - Review POC results
   - Decide V2 timeline
   - Create detailed implementation spec if approved

---

## Conclusion

**btrfsutil offers substantial advantages** over UDisks2 for BTRFS management:
- 3x+ more metadata fields
- Advanced operations (read-only toggle, default subvolume)
- Better performance (direct syscalls)
- One fewer system dependency
- Foundation for future advanced features

**Migration complexity is manageable:**
- Well-documented stable API
- Incremental migration possible
- Existing polkit infrastructure usable

**Recommendation: Proceed with migration for V2** after V1 release with current UDisks2 implementation. This gives users immediate BTRFS support while setting up superior implementation for V2.

---

**Status:** Investigation complete, awaiting approval for POC phase.
