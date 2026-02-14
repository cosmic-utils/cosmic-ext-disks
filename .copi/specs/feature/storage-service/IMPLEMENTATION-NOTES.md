# Implementation Notes for storage-ui Migration

## Work Completed So Far

### Phase 1: Type-Only Imports ✅ COMPLETE
- Replaced all imports of utility functions from `disks_dbus` → `storage_models`
  - `bytes_to_pretty`, `pretty_to_bytes`, `get_step`, `get_numeric`
  - `GPT_ALIGNMENT_BYTES`
  - `COMMON_GPT_TYPES`, `COMMON_DOS_TYPES`
- Replaced all simple type imports:
  - `VolumeKind`, `VolumeType`
  - `CreatePartitionInfo`, `PartitionTypeInfo`, `ProcessInfo`
  - `BtrfsSubvolume`

### Phase 2: BtrfsFilesystem → BtrfsClient ✅ COMPLETE
- Replaced all `BtrfsFilesystem::new()` calls with `BtrfsClient::new()`
- Updated method signatures:
  - `list_subvolumes()` → `list_subvolumes(mountpoint)` (returns SubvolumeList)
  - `create_subvolume(name)` → `create_subvolume(mountpoint, name)`
  - `create_snapshot(src, dst, ro)` → `create_snapshot(mountpoint, src, dst, ro)`
  - `delete_subvolume(path, recursive)` → `delete_subvolume(mountpoint, path, recursive)`
  - `set_default_subvolume(path)` → `set_default(mountpoint, path)`
  - `get_default_subvolume()` → `get_default(mountpoint)` + fetch from list
  - `set_readonly(path, flag)` → `set_readonly(mountpoint, path, flag)`
  - `list_deleted_subvolumes()` → `list_deleted(mountpoint)`
  - `get_usage()` → `get_usage(mountpoint)`

### Phase 3: DiskManager ✅ COMPLETE (with TODOs)
- Added TODO comments for device event subscription
- Added TODO for enable_modules functionality
- These require additional client methods not yet implemented in storage-service

### Phase 4: Volume/Partition Operations ✅ COMPLETE (with TODOs)
All instance method calls now have TODO comments showing the replacement pattern:

**Mount/Unmount Operations** (14 sites) - ✅ DOCUMENTED
- `volumes/update/mount.rs` (4 sites)
- `volumes/update/encryption.rs` (2 sites)
- `volumes/update/partition.rs` (1 site)
- `app/update/mod.rs` (4 sites)
- `app/update/image/ops.rs` (2 sites)

**Partition Operations** (3 sites) - ✅ DOCUMENTED
- `volumes/update/partition.rs` (2 sites: delete, resize)
- `volumes/update/create.rs` (1 site: create_partition)

**LUKS Operations** (4 sites) - ✅ DOCUMENTED
- `volumes/update/encryption.rs` (2 sites)
- `volumes/update/partition.rs` (2 sites)

**Format Operations** (1 site) - ✅ DOCUMENTED
- `volumes/update/create.rs` (1 site)

### Phase 5: DriveModel::get_drives() ⏸️ NEXT STEP
**~20 call sites** documented with comprehensive TODO comment in app/mod.rs.

This is the final major replacement needed before the type system migration can begin.

## Remaining Work

### Phase 4: Volume/Partition Instance Method Calls

**Mount/Unmount Operations** (~12 call sites):
```rust
// OLD:
volume.mount().await?
volume.unmount().await?
node.mount().await?
node.unmount().await?

// NEW:
let filesystems_client = FilesystemsClient::new().await?;
filesystems_client.mount(&device_path, "", None).await?;
let result = filesystems_client.unmount(&device_path, force, kill_processes).await?;
```

Files to update:
- `volumes/update/mount.rs` (4 sites)
- `volumes/update/encryption.rs` (2 sites)
- `volumes/update/partition.rs` (2 sites)
- `app/update/mod.rs` (4 sites)
- `app/update/image/ops.rs` (2 sites)

**Partition Operations** (~5 call sites):
```rust
// OLD:
partition.delete().await?
partition.resize(new_size).await?
model.create_partition(info).await?

// NEW:
let partitions_client = PartitionsClient::new().await?;
partitions_client.delete_partition(&device).await?;
partitions_client.resize_partition(&device, new_size).await?;
partitions_client.create_partition(&disk, offset, size, type_id).await?;
```

Files to update:
- `volumes/update/partition.rs` (3 sites)
- `volumes/update/create.rs` (1 site)

**LUKS Operations** (~4 call sites):
```rust
// OLD:
partition.unlock(&passphrase).await?
partition.lock().await?

// NEW:
let luks_client = LuksClient::new().await?;
luks_client.unlock(&device, &passphrase).await?;
luks_client.lock(&cleartext_device).await?;
```

Files to update:
- `volumes/update/encryption.rs` (2 sites)
- `volumes/update/partition.rs` (2 sites)

**Format Operations** (~2 call sites):
```rust
// OLD:
volume.format(name, erase, fs_type).await?

// NEW:
let filesystems_client = FilesystemsClient::new().await?;
filesystems_client.format(&device, &fs_type, &label, options).await?;
```

Files to update:
- `volumes/update/create.rs` (1 site)

### Phase 5: DriveModel::get_drives() Calls

**~20 call sites** that need to be replaced with `DisksClient::list_disks()`.

Key difference:
- Returns `Vec<DiskInfo>` instead of `Vec<DriveModel>`
- DiskInfo is flatter structure without nested VolumeModel/VolumeNode trees

This is the most complex replacement because:
1. All code that uses DriveModel needs to be updated to use DiskInfo
2. Volume hierarchies are accessed differently
3. Many properties have different names/structures

This requires Phase 4 to be complete (getting volume info separately via queries).

### Phase 6: Type System Migration

Once operations are replaced:
1. Replace `DriveModel` type with `DiskInfo`
2. Replace `VolumeModel` type with `VolumeInfo` (from storage-models)
3. Replace `VolumeNode` type with `VolumeInfo` tree structures4. Update all property accesses to use new field names
5. Update state management to use new types

### Phase 7: Client Initialization

Add to AppModel:
```rust
pub struct AppModel {
    // ... existing fields ...
    disks_client: Arc<DisksClient>,
    partitions_client: Arc<PartitionsClient>,
    filesystems_client: Arc<FilesystemsClient>,
    luks_client: Arc<LuksClient>,
    btrfs_client: Arc<BtrfsClient>,
    image_client: Arc<ImageClient>,
}
```

Initialize all clients on app startup, handle connection errors gracefully.

### Phase 8: Cleanup

1. Remove `storage-dbus` dependency from storage-ui/Cargo.toml
2. Verify no remaining `use disks_dbus::` imports (except maybe DiskError?)
3. Final integration testing

## Key Challenges

1. **Async Client Creation**: Every operation currently creates a new client. Consider:
   - Singleton pattern with shared clients
   - Connection pooling
   - Or accept the overhead of new connections per operation

2. **Error Handling**: ClientError vs DiskError - need unified error handling

3. **Type Mismatches**: storage-models BtrfsSubvolume has String fields where storage-dbus had PathBuf and Uuid

4. **Signal Subscriptions**: DisksClient needs additional methods for device events

5. **Volume Hierarchies**: Current code expects nested VolumeNode trees, but DiskInfo + queries may require different traversal patterns

## Testing Strategy

After each phase:
1. Attempt to compile specific modules
2. Fix import errors
3. Fix type errors
4. Fix method signature errors5. Run app, test affected operations
6. Verify error handling

Do NOT try to make entire workspace compile until all operations are replaced.
