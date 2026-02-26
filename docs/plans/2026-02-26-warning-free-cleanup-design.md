# Warning-Free Cleanup Design (Service/Client Focus)

## Summary
Make `cargo clippy --workspace --all-targets` warning-free by pruning unused service/client APIs that have no active call path, while preserving intentionally staged UI helpers.

## Current Flow Assessment

### How the app currently works
- The app uses direct, task-scoped client calls in update handlers and subscriptions.
- Data refresh is mostly full-refresh/event-driven (signals + reload), not atomic per-entity mutation.
- Network flows rely on `list_remotes` and CRUD/test/mount operations; several read-only convenience methods are unused.

### Is the current way "best" right now?
- For this codebase state, yes: direct client calls from update modules are clear and avoid premature abstraction.
- The warning set indicates API surface drift (extra methods/types) rather than missing architecture.
- Therefore the best immediate move is to shrink API surface to what is exercised today, not add synthetic call sites.

## Approaches Considered

### Approach A: Hard prune everything dead
- Remove all dead code (including UI staging helpers).
- Pros: smallest codebase, no dead symbols.
- Cons: loses UI scaffolding and may slow near-term UI iteration.

### Approach B: Hybrid prune (recommended)
- Remove dead service/client and transport-adjacent API surface with no active alternative.
- Keep intentionally staged UI helpers, but isolate/scope dead-code allowances to UI-only modules.
- Pros: warning-free, minimal risk, preserves planned UI scaffolding.
- Cons: retains some dormant UI code by design.

### Approach C: Keep APIs and force usage
- Add synthetic usages/tests solely to silence warnings.
- Pros: no API removals.
- Cons: adds noise and obscures real production flow.

## Recommendation
Use **Approach B**.

## Decision List (by warning group)

### Remove now (service/client surface with no active alternative)
- `storage-app/src/client/disks.rs`
  - Remove `get_volume_info`
  - Remove `eject`
- `storage-app/src/client/filesystems.rs`
  - Remove `FilesystemUsage`
  - Remove `list_filesystems`
  - Remove `get_supported_filesystems`
  - Remove `get_blocking_processes`
  - Remove `get_usage`
- `storage-app/src/client/image.rs`
  - Remove `list_active_operations`
  - Trim `OperationStatus` fields to currently-read fields (`bytes_completed`, `total_bytes`, `speed_bytes_per_sec`) and any required identity field if call sites need it
- `storage-app/src/client/luks.rs`
  - Remove `list_encrypted_devices`
  - Remove `format`
- `storage-app/src/client/lvm.rs`
  - Remove entire `LvmClient` module from `storage-app` (and re-export in `client/mod.rs`)
- `storage-app/src/client/partitions.rs`
  - Remove `create_partition` (use `create_partition_with_filesystem` path)
- `storage-app/src/client/rclone.rs`
  - Remove `get_remote`
  - Remove `supported_remote_types`

### Remove now (stale duplicate UI type, no active path)
- `storage-app/src/ui/btrfs/message.rs`
  - Remove unused local enum/module if `app::message` is the canonical message surface.

### Keep (unused UI scaffolding by intent)
- `storage-app/src/models/ui_drive.rs` helper methods
- `storage-app/src/models/ui_volume.rs` helper methods
- `storage-app/src/ui/network/state.rs` sort/filter helper methods
- `storage-app/src/ui/volumes/state.rs` `Segment.partition_type`
- `storage-app/src/utils/ui.rs` info/alert helper family

### How to keep UI scaffolding and still be warning-free
- Reintroduce **targeted** `#[allow(dead_code)]` at the smallest practical scope for UI-only staging code.
- Do **not** use crate-level allowances.
- Add short doc comments marking these as "reserved for upcoming UI integration".

## Execution Plan

1. Prune non-UI dead client APIs and dependent types.
2. Remove stale BTRFS local message module if fully redundant.
3. Apply scoped dead-code allowances only to retained UI scaffolding.
4. Run `cargo clippy --workspace --all-targets` and iterate until zero warnings.
5. Validate with `cargo check --workspace`.

## Risk Notes
- API removals are internal to `storage-app` and low risk if no call sites exist.
- Removing `LvmClient` should be done in one commit with module export updates to avoid temporary compile drift.
- UI scaffolding allowances must stay scoped so dead code does not creep back into service/client layers.
