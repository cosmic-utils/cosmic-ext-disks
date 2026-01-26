# chore/audit-2026-01-25-architecture-cleanup — Tasks

Branch: `chore/audit-2026-01-25-architecture-cleanup`

Source audit: `.copi/audits/2026-01-25T23-24-44Z.md`

This is a refactor-only track. Each task should be a small PR (or one squash-merge commit) that keeps `cargo fmt`, `cargo clippy`, and `cargo test` green.

## Task 1: Create new UI module skeleton (no behavior change)

- Scope: Establish `disks-ui/src/ui/` hierarchy for app/dialogs/volumes.
- Files/areas:
  - `disks-ui/src/ui/mod.rs` (new)
  - `disks-ui/src/ui/app/{mod.rs,state.rs,message.rs,update.rs,view.rs,subscriptions.rs}` (new)
- Steps:
  - Add `ui` module tree and re-export from existing `app.rs` temporarily.
  - Move only type declarations first (no logic changes).
  - Keep compilation green after each move.
- Test plan:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-features`
  - `cargo test --workspace --all-features`
- Done when:
  - [x] App still builds/runs; no logic changes; new modules exist.

## Task 2: Move dialog state + messages under `ui/dialogs/`

- Scope: Resolve GAP-003 (dialog hierarchy inversion).
- Files/areas:
  - Move `ShowDialog` + dialog structs from `disks-ui/src/app.rs` → `disks-ui/src/ui/dialogs/state.rs`
  - Move dialog message enums from `disks-ui/src/app.rs` → `disks-ui/src/ui/dialogs/message.rs`
  - Update imports in dialog views.
- Steps:
  - Introduce `DialogState`/`DialogMessage` and update `Message` wrapper.
  - Ensure dialog views do not import volumes message enums.
- Test plan: standard workspace fmt/clippy/test.
- Done when:
  - `disks-ui/src/app.rs` no longer defines dialog structs.

## Task 3: Split volumes module into state/message/update/view/actions

- Scope: Resolve GAP-002 (volumes “god file”).
- Files/areas:
  - Create `disks-ui/src/ui/volumes/{mod.rs,state.rs,message.rs,update.rs,view.rs,actions.rs}`
  - Move `VolumesControl` and `VolumesControlMessage` and related message enums.
- Steps:
  - Move message enums first.
  - Move state next.
  - Move `update` handler next.
  - Move rendering helpers last.
- Test plan: standard workspace fmt/clippy/test.
- Done when:
  - Old `disks-ui/src/views/volumes.rs` becomes a small compatibility shim or is removed.

## Task 4: Reduce `AppModel` file by splitting message/state/update/view/subscriptions

- Scope: Resolve GAP-001 (app “god file”).
- Files/areas:
  - `disks-ui/src/ui/app/*`
  - Make `disks-ui/src/app.rs` a thin re-export/glue layer (or rename to `ui/app/mod.rs` and keep main pointing to it).
- Steps:
  - Move `Message` enum.
  - Move `AppModel` struct and init.
  - Move `view()` and `update()`.
  - Move subscriptions.
- Test plan: standard workspace fmt/clippy/test.
- Done when:
  - No single module exceeds ~400 LOC without justification.

## Task 5: Fix naming typos (mechanical rename)

- Scope: Resolve GAP-008.
- Files/areas:
  - `PasswordProectedUpdate` → `PasswordProtectedUpdate`
  - `selected_partitition_type` → `selected_partition_type_index` (or chosen final name)
- Steps:
  - Rename in `CreatePartitionInfo` and all UI call sites.
  - Update any persisted state or serialization if present (likely none).
- Test plan:
  - `rg "partitition|proected"` returns no results.
  - standard workspace fmt/clippy/test.
- Done when:
  - No typo identifiers remain.

## Task 6: Deduplicate bytestring/mount-point decoding helpers in `disks-dbus`

- Scope: Resolve GAP-006.
- Files/areas:
  - New helper module: `disks-dbus/src/dbus/bytestring.rs` (or `udisks/bytestring.rs`)
  - Update `VolumeModel` module (currently `disks-dbus/src/disks/partition.rs`, planned rename in Task 8) and `disks-dbus/src/disks/volume.rs` to use it.
- Steps:
  - Extract `decode_c_string_bytes`, `decode_mount_points`, and “Vec<u8> c-string” behavior.
  - Replace duplicates.
  - Add focused unit tests in `disks-dbus`.
- Test plan: standard workspace fmt/clippy/test.
- Done when:
  - Duplicated helpers removed; tests cover decode edge cases.

## Task 7: Standardize byte formatting to a single implementation

- Scope: Resolve GAP-007.
- Files/areas:
  - Remove or deprecate UI copy in `disks-ui/src/utils/format.rs`
  - Use `disks_dbus::bytes_to_pretty` in UI (DBus remains canonical; no new common crate).
- Steps:
  - Switch imports in UI.
  - Delete UI duplicate file if unused.
- Test plan: standard workspace fmt/clippy/test.
- Done when:
  - Only one `bytes_to_pretty/pretty_to_bytes/get_numeric/get_step` implementation remains.

## Task 8: Unify vocabulary: remove `PartitionModel` alias and clarify “partition vs volume”

- Scope: Resolve GAP-009.
- Files/areas:
  - Remove `pub type PartitionModel = VolumeModel` from `disks-dbus/src/disks/mod.rs`.
  - Rename the file/module that defines `VolumeModel` to match its name/role:
    - from `disks-dbus/src/disks/partition.rs`
    - to `disks-dbus/src/disks/volume_model.rs` (or `disks-dbus/src/disks/volume_model/mod.rs`).
  - Update all imports/exports/call sites to use `VolumeModel` directly.
- Steps:
  - Rename module file and update `mod.rs` wiring.
  - Remove the alias export.
  - Run a repo-wide `rg "PartitionModel"` and eliminate remaining uses.
- Test plan: standard workspace fmt/clippy/test.
- Done when:
  - `PartitionModel` no longer exists anywhere; `VolumeModel` is consistently used.

## Task 9: Split `DriveModel` by responsibility (discovery/actions/smart/tree)

- Scope: Resolve GAP-004.
- Files/areas:
  - `disks-dbus/src/disks/drive/` submodules.
- Steps:
  - Move discovery routines first.
  - Move SMART next.
  - Move format/remove/eject actions next.
  - Move volume tree builder last.
- Test plan: standard workspace fmt/clippy/test.
- Done when:
  - Drive code is split and readable; public API remains stable.

## Task 10: Partition type catalog refactor

- Scope: Resolve GAP-005.
- Files/areas:
  - `disks-dbus/src/partition_type.rs` → `disks-dbus/src/partition_types/{mod.rs,gpt.rs,dos.rs}`
- Steps:
  - Split GPT and DOS data into separate modules to reduce file size.
  - Ensure lookup functions remain unchanged.
  - Optional follow-up task: move data into TOML/JSON and parse at build-time.
- Test plan:
  - Add/keep minimal unit tests to verify counts and a few known IDs map.
- Done when:
  - `partition_type.rs` no longer contains the full giant array.

## Task 11: Remove unwrap-based crash paths in UI view

- Scope: Resolve GAP-010.
- Files/areas:
  - `disks-ui/src/ui/app/view.rs`
  - `disks-ui/src/ui/volumes/state.rs`
- Steps:
  - Make selection state self-healing (clamp selection; handle empty segments).
  - Replace unwraps with fallbacks (Info dialog or empty-state view).
- Test plan:
  - Manual: hotplug loop device while app open; no panics.
  - standard workspace fmt/clippy/test.
- Done when:
  - No unwraps remain in `view()` for nav/segment selection.

## Task 12: Logging layering cleanup

- Scope: Resolve GAP-011.
- Files/areas:
  - UI: replace scattered `println!/eprintln!` with consistent approach.
- Steps:
  - Decide: use `tracing` (preferred) + user-visible Info dialogs.
  - Convert anomaly logs to `tracing::warn!` and only show UI dialog for actionable errors.
- Test plan: standard workspace fmt/clippy/test.
- Done when:
  - Logging is consistent and layered.
