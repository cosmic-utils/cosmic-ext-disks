# Research: Storage-UI Architecture Analysis

**Date**: 2026-02-14
**Purpose**: Identify overcomplexity, poor conventions, and improvement opportunities in `storage-ui` crate

## Executive Summary

The storage-ui crate follows the COSMIC application pattern correctly but exhibits complexity issues in message routing and state management. The primary concerns are:

1. **Deep message wrapping chains** requiring 15+ `From` trait implementations
2. **Large state structs** with nested subcomponents
3. **Update handler files** that are individually large (10-20KB each)

## Findings

### 1. Message Routing Complexity

**Location**: `storage-ui/src/ui/volumes/message.rs`

**Current State**:
- 49-line enum with 29 variants
- 15+ `From<T> for VolumesControlMessage` implementations
- 6+ `From<T> for Message` implementations (via VolumesControlMessage)

**Pattern Observed**:
```
DialogMessage → VolumesControlMessage → Message
```

**Issues**:
- Each dialog message type requires two From impls (one for VolumesControlMessage, one for Message)
- Message.rs file is mostly boilerplate conversions (lines 51-187)
- Adding new dialogs requires touching multiple message files

**Recommendation**:
- Consider a macro to generate From impls automatically
- Or use a generic `DialogMessage<D>` wrapper pattern

---

### 2. App-Level Message Enum Size

**Location**: `storage-ui/src/ui/app/message.rs`

**Current State**:
- 136 lines
- 60+ message variants
- Includes: navigation, dialogs, BTRFS operations, image operations, sidebar, etc.

**Issues**:
- Single monolithic enum makes it hard to track which messages affect which state
- Mixing concerns: UI navigation, async operations, and dialog state

**Recommendation**:
- Consider grouping related messages into sub-enums
- Example: `Message::Btrfs(BtrfsMessage)`, `Message::Dialog(DialogMessage)`

---

### 3. State Struct Complexity

**Location**: `storage-ui/src/ui/volumes/state.rs`

**Current State**:
- 434 lines
- `VolumesControl` struct with 13 fields
- `Segment` struct with 11 fields
- Complex segment computation logic (lines 150-333)

**Issues**:
- `get_segments()` method is 183 lines long
- Segment computation handles edge cases for GPT, DOS, alignment, merging
- State struct mixes UI concerns (selected_segment) with data (partitions, volumes)

**Recommendation**:
- Extract segment computation to a separate module: `utils/segments.rs` (already exists)
- Consider splitting VolumesControl into data + UI state

---

### 4. Update Handler Organization

**Location**: `storage-ui/src/ui/volumes/update/`

**Files**:
| File | Size | Purpose |
|------|------|---------|
| btrfs.rs | 8KB | BTRFS subvolume/snapshot dialogs |
| create.rs | 9KB | Partition creation |
| encryption.rs | 21KB | LUKS operations, unlock, passphrase |
| filesystem.rs | 9KB | Label editing, filesystem check/repair |
| mount_options.rs | 13KB | Mount options dialog |
| mount.rs | 10KB | Mount/unmount operations |
| partition.rs | 14KB | Partition edit/resize/delete |
| selection.rs | 8KB | Segment/volume selection |

**Issues**:
- `encryption.rs` is 21KB - handles unlock, lock, passphrase change, ownership
- Files are individually large but well-organized by domain
- Each file handles both dialog state and async operations

**Recommendation**:
- Split `encryption.rs` into `luks.rs` and `ownership.rs`
- Consider extracting dialog state management to separate files

---

### 5. COSMIC Convention Adherence

**Analysis**: The crate correctly follows COSMIC patterns:

| Pattern | Implementation | Status |
|---------|---------------|--------|
| Application trait | `storage-ui/src/ui/app/mod.rs` | ✅ Correct |
| Core state management | AppModel with Core field | ✅ Correct |
| Task-based updates | Returns `Task<Message>` | ✅ Correct |
| Subscriptions | `subscriptions.rs` for async events | ✅ Correct |
| Context drawers | ContextPage enum | ✅ Correct |
| Nav bar integration | nav_bar::Model | ✅ Correct |

**Issues**:
- Custom sidebar replaces nav_bar - this is intentional for tree view
- Dialog management uses custom ShowDialog enum instead of COSMIC dialog pattern

---

### 6. Code Duplication Patterns

**Location**: Multiple files

**Found**:
1. `flatten_volumes()` function duplicated in `state.rs` (lines 339-343 and 406-410)
2. Similar async command patterns repeated across update handlers
3. Dialog opening pattern repeated: set dialog state, return Task

**Recommendation**:
- Extract `flatten_volumes` to `models/helpers.rs`
- Create helper macro for dialog opening pattern

---

## Priority Recommendations

### High Priority (Address Soon)

1. **Extract flatten_volumes** - Simple fix, reduces duplication
2. **Split encryption.rs** - 21KB file is hard to navigate

### Medium Priority (Plan for Next Sprint)

3. **Message routing macro** - Reduces boilerplate for new dialogs
4. **Segment computation extraction** - Move complex logic to utils

### Low Priority (Future Refactoring)

5. **Message enum grouping** - Requires broader refactoring
6. **State struct splitting** - Consider as part of larger UI reorganization

---

## Metrics Summary

| Metric | Value | Assessment |
|--------|-------|------------|
| Total source files | 70 | Moderate |
| Largest file | encryption.rs (21KB) | Too large |
| Message enum variants | 60+ | High |
| From impls in message.rs | 15+ | High |
| State struct fields | 13 (VolumesControl) | Moderate |
| Update handler files | 8 (volumes) + 5 (app) | Good organization |

---

## Additional Module Analyses

### 7. Dialogs Module Analysis

**Location**: `storage-ui/src/ui/dialogs/`

**Files**:
| File | Lines | Purpose |
|------|-------|---------|
| mod.rs | 3 | Module exports |
| message.rs | 171 | Dialog message types (25+ variants) |
| state.rs | 233 | Dialog state management (ShowDialog enum) |

**Observations**:
- Well-organized with clear message/state separation
- `ShowDialog` enum provides type-safe dialog management
- Dialog messages are numerous but logically grouped

**Status**: ✅ No issues - follows good patterns

---

### 8. BTRFS Module Analysis

**Location**: `storage-ui/src/ui/btrfs/`

**Files**:
| File | Lines | Purpose |
|------|-------|---------|
| mod.rs | 6 | Module exports |
| message.rs | 32 | BTRFS-specific messages |
| state.rs | 51 | BTRFS management state (subvolumes, usage) |
| view.rs | 300 | BTRFS UI rendering |

**Observations**:
- Clean separation of concerns (message/state/view)
- view.rs is 300 lines but handles complex BTRFS display
- State is simple (subvolumes list, loading flags)

**Status**: ✅ No issues - well-structured

---

### 9. Sidebar Module Analysis

**Location**: `storage-ui/src/ui/sidebar/`

**Files**:
| File | Lines | Purpose |
|------|-------|---------|
| mod.rs | 4 | Module exports |
| state.rs | 54 | Sidebar tree state |
| view.rs | 509 | Custom treeview rendering |

**Observations**:
- Custom treeview implementation replaces COSMIC nav_bar
- view.rs is largest file (509 lines) - handles complex tree rendering
- State is minimal (expanded nodes, selection)

**Recommendation**:
- Consider extracting tree rendering helpers from view.rs

**Status**: ⚠️ view.rs is large but acceptable for custom component

---

### 10. Models Module Analysis

**Location**: `storage-ui/src/models/`

**Files**:
| File | Lines | Purpose |
|------|-------|---------|
| mod.rs | ~20 | Module exports |
| helpers.rs | ~100 | Helper functions |
| load.rs | ~150 | Async data loading |
| ui_drive.rs | ~100 | Drive UI model |
| ui_volume.rs | ~150 | Volume UI model |

**Observations**:
- Clean domain model separation
- `load.rs` handles async data fetching
- Models are simple wrappers around storage_models types

**Status**: ✅ No issues - clean architecture

---

### 11. Client Module Analysis

**Location**: `storage-ui/src/client/`

**Files**:
| File | Lines | Purpose |
|------|-------|---------|
| mod.rs | 19 | Client exports |
| btrfs.rs | ~150 | BTRFS D-Bus client |
| disks.rs | ~200 | Disk operations client |
| error.rs | ~50 | Client error types |
| filesystems.rs | ~200 | Filesystem client |
| image.rs | 217 | Disk image operations |
| luks.rs | 149 | LUKS encryption client |
| lvm.rs | 190 | LVM client |
| partitions.rs | 159 | Partition client |

**Observations**:
- Well-organized by domain (matches storage-dbus structure)
- Each client module is reasonably sized (<220 lines)
- Clear separation between UI and D-Bus communication

**Status**: ✅ No issues - follows good patterns

---

## Complete Module Coverage (FR-006)

| Module | Analyzed | Issues Found |
|--------|----------|--------------|
| ui/app | ✅ Yes | Message enum size, update handler organization |
| ui/volumes | ✅ Yes | Message routing, state complexity, encryption.rs size |
| ui/dialogs | ✅ Yes | None |
| ui/btrfs | ✅ Yes | None |
| ui/sidebar | ✅ Yes | view.rs size (acceptable) |
| models | ✅ Yes | None |
| client | ✅ Yes | None |

---

## Final Recommendations Summary

### Immediate Actions (High ROI)

1. **Extract `flatten_volumes()`** from `state.rs` to `models/helpers.rs` - eliminates duplication
2. **Split `encryption.rs`** into `luks.rs` and `ownership.rs` - improves navigation

### Future Considerations

3. **Add macro for From impls** - reduces boilerplate when adding new dialogs
4. **Extract segment computation** from `state.rs` to `utils/segments.rs`
5. **Consider message enum grouping** - would require broader refactoring
