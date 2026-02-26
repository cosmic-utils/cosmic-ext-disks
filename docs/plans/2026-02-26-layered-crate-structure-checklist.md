# Layered Crate Structure Migration Checklist

**Date:** 2026-02-26  
**Scope:** Detailed actionable checklist from approved brainstorming session.

---

## A. Preparation

- [x] Confirm branch scope remains structural-only (no feature work bundled).
- [x] Capture baseline output of `just check`.
- [x] 2026-02-26T21:29:33+00:00 baseline re-check for logical-volume planning: `just check` passed (`clippy`, `fmt --check`, `test --no-run`).
- [x] Record current module trees per crate for before/after comparison.
- [x] Define naming glossary (`disk/disks`, `filesystem/filesystems`, `encryption/luks`) for consistency.

## B. Global Guardrails

- [x] Preserve `storage-app` layered split (`message/state/update/views/controls`).
- [x] Preserve `storage-service` layered split (`handlers/policies/utilities`).
- [x] Prevent new root-level catch-all modules (`helpers/common/misc`) unless justified.
- [x] Keep `main.rs`/`lib.rs` focused on wiring + exports, not heavy logic.

## C. Crate-by-Crate Structural Tasks

## C1. `storage-types`

- [x] Keep module name `partition_types` unchanged (no rename to `partition_catalog`).
- [x] Enforce exact module declaration order in `storage-types/src/lib.rs`:
	`common`, `caller`, `disk`, `partition`, `partition_types`, `filesystem`, `encryption`, `lvm`, `btrfs`, `rclone`, `smart`, `usage_scan`, `volume`.
- [x] Convert file to folder module without changing public module paths:
	`storage-types/src/partition_types.rs` → `storage-types/src/partition_types/mod.rs`.
- [x] Create partition catalog internals:
	`storage-types/src/partition_types/catalog.rs` (TOML constants + `LazyLock` data),
	`storage-types/src/partition_types/query.rs` (`find_by_id`, `get_valid_partition_names`, `get_all_partition_type_infos`).
- [x] Convert `storage-types/src/rclone.rs` to folder module form:
	`storage-types/src/rclone.rs` → `storage-types/src/rclone/mod.rs`.
- [x] Create focused rclone internals:
	`storage-types/src/rclone/scope.rs` (`ConfigScope` + path/prefix helpers),
	`storage-types/src/rclone/remote.rs` (`RemoteConfig`, `NetworkMount`, `RemoteConfigList`),
	`storage-types/src/rclone/provider_catalog.rs` (`RcloneProvider*`, providers JSON loading, lookup fns),
	`storage-types/src/rclone/mount.rs` (`MountStatus`, `MountType`),
	`storage-types/src/rclone/results.rs` (`TestResult`, `MountStatusResult`).
- [x] In `storage-types/src/lib.rs`, replace wildcard re-exports with explicit grouped exports; keep crate-root API for currently consumed symbols (`COMMON_GPT_TYPES`, `COMMON_DOS_TYPES`, `get_all_partition_type_infos`, `get_valid_partition_names`, `make_partition_flags_bits`, `ConfigScope`, `Usage`, `ByteRange`, etc.).
- [x] Remove dead `PartitionTypeInfo::find_by_id` only if final grep confirms no in-workspace references after C1 rewiring.

## C2. `storage-contracts`

- [x] Rename files under `storage-contracts/src/protocol/`:
	`errors.rs` → `error.rs`, `ids.rs` → `id.rs`, `operations.rs` → `operation.rs`.
- [x] In `storage-contracts/src/protocol/mod.rs`, change module declarations exactly to:
	`pub mod error;`, `pub mod id;`, `pub mod operation;` (in that order).
- [x] In `storage-contracts/src/protocol/mod.rs`, change re-exports exactly to:
	`pub use error::{StorageError, StorageErrorKind};`
	`pub use id::OperationId;`
	`pub use operation::{OperationEvent, OperationKind, OperationProgress};`
- [x] In `storage-contracts/src/traits/discovery.rs`, rename trait `FilesystemOps` → `FilesystemDiscovery` to remove naming overlap with `FilesystemOpsAdapter`.
- [x] In `storage-contracts/src/traits/mod.rs`, update re-export exactly:
	`pub use discovery::{DiskDiscovery, FilesystemDiscovery, Partitioning};`.
- [x] In `storage-contracts/src/lib.rs`, replace wildcard façade re-exports with explicit grouped exports from `protocol` and `traits` (no `pub use protocol::*; pub use traits::*;`).
- [x] Update imports for protocol singular rename paths (`errors|ids|operations` → `error|id|operation`) and trait rename (`FilesystemOps` → `FilesystemDiscovery`) in any consumer code (if/when consumers are added).

## C3. `storage-udisks`

- [x] Create module folder `storage-udisks/src/infra/` with `mod.rs`.
- [x] Move files:
	`storage-udisks/src/options.rs` → `storage-udisks/src/infra/options.rs`
	`storage-udisks/src/usage.rs` → `storage-udisks/src/infra/usage.rs`
	`storage-udisks/src/udisks_block_config.rs` → `storage-udisks/src/infra/udisks_block_config.rs`
	`storage-udisks/src/util/process.rs` → `storage-udisks/src/infra/process.rs`.
- [x] Delete obsolete utility module shell `storage-udisks/src/util/mod.rs` once imports are rewired.
- [x] In `storage-udisks/src/lib.rs`, replace private module declarations exactly:
	`mod options; mod udisks_block_config; mod usage;` with `mod infra;`.
- [x] Update internal imports:
	`storage-udisks/src/filesystem/config.rs` and `storage-udisks/src/encryption/config.rs`
	from `crate::options::*` / `crate::udisks_block_config::*`
	to `crate::infra::options::*` / `crate::infra::udisks_block_config::*`.
- [x] Delete obsolete utility module shell `storage-udisks/src/util/mod.rs` and remove `pub mod util;` from `storage-udisks/src/lib.rs`.
- [x] Keep the public helper API identical by re-exporting from `infra::*` equivalents in `storage-udisks/src/lib.rs` (`join_options`, `stable_dedup`, `Usage`, `usage_for_mount_point`, `find_processes_using_mount`, `kill_processes`).
- [x] Keep `storage-udisks/src/dbus/**` untouched as DBus adapter boundary (no moves out of `dbus/`).

## C4. `storage-sys`

- [x] Keep top-level module layout unchanged for `error.rs`, `image.rs`, and `usage/**` (do not introduce a blanket `src/ops/` hierarchy).
- [x] Convert `storage-sys/src/rclone.rs` into module folder form:
	`storage-sys/src/rclone.rs` → `storage-sys/src/rclone/mod.rs`.
- [x] Create focused rclone internals:
	`storage-sys/src/rclone/mount_state.rs` (mount detection/unescape helpers),
	`storage-sys/src/rclone/unix_user.rs` (uid/gid/chown helpers),
	`storage-sys/src/rclone/systemd.rs` (systemd unit + systemctl orchestration).
- [x] Keep public API paths unchanged by re-exporting from `storage-sys/src/rclone/mod.rs`:
	`RCloneCli`, `set_mount_on_boot`, `is_mount_on_boot_enabled`.
- [x] In `storage-sys/src/usage/mod.rs`, add explicit re-exports for currently deep-imported helpers:
	`discover_local_mounts_under`, `estimate_used_bytes_for_mounts`, `compute_progress_percent`, `format_bytes`.
- [x] Update consumers to stop deep module traversal where applicable:
	`storage-sys/src/bin/scan-categories.rs` and `storage-service/src/handlers/filesystems.rs` use `storage_sys::usage::*` façade exports.
- [x] Keep `storage-sys/src/lib.rs` module declarations as `pub mod error; pub mod image; pub mod rclone; pub mod usage;` with existing top-level re-exports intact.

## C5. `storage-btrfs`

- [x] Delete dead file `storage-btrfs/src/types.rs` (it is not wired into `lib.rs` and duplicates models now centralized in `storage-types`).
- [x] In `storage-btrfs/src/lib.rs`, remove `pub use btrfsutil;` so low-level crate internals are not exposed as part of public API.
- [x] Keep normalized public surface explicit in `storage-btrfs/src/lib.rs`: `BtrfsError`, `Result`, `SubvolumeManager`, `get_filesystem_usage`, and selected shared model exports from `storage_types::btrfs`.
- [x] Replace wildcard shared-model export (`pub use storage_types::btrfs::*;`) with explicit type list used by consumers (`BtrfsSubvolume`, `FilesystemUsage`, `SubvolumeList`, `DeletedSubvolume` as needed).
- [x] Verify no workspace code depends on `disks_btrfs::btrfsutil` re-export (should be none) and update imports only if found.
- [x] Keep CLI-only dependencies behind `cli` feature and trim stale core dependencies in `storage-btrfs/Cargo.toml` that were only needed by removed `types.rs`.

## C6. `storage-macros`

- [x] Create files:
	`storage-macros/src/parse.rs`
	`storage-macros/src/transform.rs`
	`storage-macros/src/emit.rs`.
- [x] Move `AuthorizedInterfaceArgs` struct + `impl Parse` from `lib.rs` into `parse.rs`.
- [x] Move `transform_method(...)` from `lib.rs` into `transform.rs`.
- [x] Keep `#[proc_macro_attribute] pub fn authorized_interface(...)` in `lib.rs` and make it call into `parse` + `transform` (and `emit` if used).
- [x] Keep compile errors text identical:
	`"#[authorized_interface] only supports async methods"`
	`"#[authorized_interface] requires method to have #[zbus(connection)] and #[zbus(header)] parameters"`.
- [x] Preserve parameter detection behavior exactly in macro internals:
	support both `#[zbus(connection)]` / `#[zbus(header)]` attributes and fallback names
	`connection|_connection`, `header|_header`.
- [x] Ensure no new public symbols beyond macro entrypoint are exposed from `lib.rs`.

## C7. `storage-service` (improvements only)

- [x] Rename handler files:
	`storage-service/src/handlers/disks.rs` → `storage-service/src/handlers/disk.rs`
	`storage-service/src/handlers/filesystems.rs` → `storage-service/src/handlers/filesystem.rs`
	`storage-service/src/handlers/partitions.rs` → `storage-service/src/handlers/partition.rs`.
- [x] Rename policy files:
	`storage-service/src/policies/disks.rs` → `storage-service/src/policies/disk.rs`
	`storage-service/src/policies/filesystems.rs` → `storage-service/src/policies/filesystem.rs`
	`storage-service/src/policies/partitions.rs` → `storage-service/src/policies/partition.rs`.
- [x] Update module declarations in:
	`storage-service/src/handlers/mod.rs` (`disks/filesystems/partitions` → `disk/filesystem/partition`)
	`storage-service/src/policies/mod.rs` (`disks/filesystems/partitions` → `disk/filesystem/partition`).
- [x] Rename handler type names and imports consistently:
	`DisksHandler` → `DiskHandler`
	`FilesystemsHandler` → `FilesystemHandler`
	`PartitionsHandler` → `PartitionHandler`
	and update all `use` lines in `storage-service/src/main.rs`.
- [x] Normalize acronym naming in handler type:
	`LVMHandler` → `LvmHandler` and update imports/usages in `main.rs` and `handlers/lvm.rs`.
- [x] Remove utility→handler layer inversion in hotplug monitor:
	move `storage-service/src/utilities/udisks.rs` → `storage-service/src/handlers/disk/hotplug.rs`
	and remove `use crate::handlers::disks::DisksHandler` dependency from utilities layer.
- [x] In `storage-service/src/main.rs`, update startup call from
	`utilities::udisks::monitor_hotplug_events(...)` to handler-owned hotplug entrypoint.
- [x] Decompose oversized handler modules into folder modules (keep DBus interface names unchanged):
	`storage-service/src/handlers/filesystem.rs` → `storage-service/src/handlers/filesystem/{mod.rs,query.rs,mount.rs,format.rs,usage.rs,ownership.rs}`
	`storage-service/src/handlers/rclone.rs` → `storage-service/src/handlers/rclone/{mod.rs,query.rs,mount.rs,config.rs,authz.rs}`.
- [x] Collapse filesystems-specific helper drift by moving utility files under filesystem handler scope:
	`storage-service/src/utilities/filesystem.rs` → `storage-service/src/handlers/filesystem/support/fs_permissions.rs`
	`storage-service/src/utilities/uid.rs` → `storage-service/src/handlers/filesystem/support/uid_groups.rs`
	`storage-service/src/utilities/usage.rs` → `storage-service/src/handlers/filesystem/support/usage_threads.rs`.
- [x] Remove duplicated caller/auth plumbing in rclone handler:
	delete local `get_caller_uid(...)` helper and use macro-injected `caller.uid` where UID is required.
	Keep secondary `crate::auth::check_authorization(...)` only where it enforces additional elevated actions.
- [x] Keep DBus object paths unchanged in `main.rs`:
	`/org/cosmic/ext/Storage/Service/disks`
	`/org/cosmic/ext/Storage/Service/filesystems`
	`/org/cosmic/ext/Storage/Service/partitions`.
- [x] Keep `handlers/service.rs` as intentional naming exception (do not rename).

## C8. `storage-app` unified cleanup (includes prior C9)

- [x] Move `src/volumes/disk_header.rs` → `src/views/disk.rs`.
- [x] Move `src/volumes/usage_pie.rs` → `src/controls/usage_pie.rs` and update all consumers in `views/app.rs`, `views/btrfs.rs`, and `views/disk.rs`.
- [x] Split `src/volumes/helpers.rs` by ownership:
	- volume tree lookup + segment lookup helpers (`find_volume_in_ui_tree`, `find_volume_for_partition`, `find_segment_for_volume`) → `src/state/volumes.rs`
	- BTRFS detection helpers (`detect_btrfs_in_node`, `detect_btrfs_for_volume`) → `src/state/btrfs.rs`
	- update-only helper (`collect_mounted_descendants_leaf_first`) → `src/update/volumes/helpers.rs`
	- partition type mapping helpers (`common_partition_filesystem_type`, `common_partition_type_index_for`) → `src/utils/partition_types.rs`.
- [x] In `src/update/mod.rs`, delete local duplicate tree helpers (`find_segment_for_volume`, `find_volume_in_tree`) and use `state::volumes` helper API.
- [x] Add one canonical disk-usage aggregation helper in `src/state/volumes.rs` for LUKS-container child usage rollup; remove duplicated aggregation logic from `src/views/app.rs` and `src/views/disk.rs`.
- [x] Centralize network/provider icon mapping into `src/controls/icons.rs`:
	move `src/network/icons.rs` content and also move scope badge helpers (`scope_icon`, `scope_label`) from `src/views/network.rs` into `controls/icons.rs`.
- [x] Convert `src/update/image.rs` into `src/update/image/mod.rs` (single folder-module form).
- [x] In `src/update/image/mod.rs`, remove pass-through wrappers and expose focused functions from `dialogs.rs` / `ops.rs`.
- [x] In `src/update/image/mod.rs`, deduplicate repeated `ImageOperationDialog` construction by introducing one local builder/helper used by create/restore (disk + partition) entrypoints.
- [x] Update `src/update/mod.rs` dispatch call sites to use final `image` module API directly (no legacy wrappers).
- [x] Remove obsolete top-level modules after rewires:
	`src/volumes/mod.rs`, `src/network/mod.rs`, `mod volumes;` and `mod network;` in `src/main.rs`.
- [x] Keep current layered architecture style; do not reintroduce top-level feature-domain partitioning.

## D. Verification Tasks

- [x] After each crate wave, run `just check` and record pass/fail plus changed crate scope.
- [x] NO temporary compatibility re-exports added during migration. If breakage occurs, fix now if not fixed in later waves.
- [x] Before final wave closure, run targeted grep for stale module paths (`volumes::`, `network::`, moved helper paths).
- [x] Run final `just check` at workspace root with zero new warnings/errors introduced by restructure.
- [x] After completing each Cx section (C1, C2, …), immediately update this checklist section status before moving to the next Cx section.

## E. PR Hygiene

- [x] Execute this migration as a single big-bang change set and create exactly one implementation commit.
- [x] Do not create intermediate commits for crate slices; keep progress tracked in this checklist during execution.
- [x] Keep structural changes free of unrelated feature work.
- [x] Update `docs/plans` status notes with what was completed before creating the final commit.

## F. Done Definition

- [x] Structures are consistently layered (with explicit app/service exceptions).
- [x] No anti-pattern outlier directories remain unexplained.
- [x] Public exports are clear and minimal.
- [x] Workspace passes verification (`just check`).
