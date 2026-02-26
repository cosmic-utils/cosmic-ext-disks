# UI Componentisation Checklist (Progress Audit)

Date: 2026-02-26
Branch: 069-polish
Reference plan: [docs/plans/2026-02-26-ui-componentisation-implementation-plan.md](docs/plans/2026-02-26-ui-componentisation-implementation-plan.md)

## Overall Status

- Tasks completed: 6 / 6
- Tasks partially completed: 0 / 6
- Current blockers: none
- Current verification: `cargo clippy --workspace --all-targets` ✅, `cargo test --workspace --no-run` ✅

## Task-by-Task Checklist

### Task 1 — Scaffold controls layer

- [x] Create controls module tree and exports
  - Evidence: [storage-app/src/controls/mod.rs](storage-app/src/controls/mod.rs)
- [x] Wire controls module at crate root
  - Evidence: [storage-app/src/main.rs](storage-app/src/main.rs)
- [x] Add all planned control files
  - Evidence:
    - [storage-app/src/controls/layout.rs](storage-app/src/controls/layout.rs)
    - [storage-app/src/controls/fields.rs](storage-app/src/controls/fields.rs)
    - [storage-app/src/controls/actions.rs](storage-app/src/controls/actions.rs)
    - [storage-app/src/controls/status.rs](storage-app/src/controls/status.rs)
    - [storage-app/src/controls/form.rs](storage-app/src/controls/form.rs)
    - [storage-app/src/controls/wizard.rs](storage-app/src/controls/wizard.rs)

Status: **Complete**

---

### Task 2 — Move reusable helpers to controls

- [x] Move wizard primitives into controls
  - Evidence: [storage-app/src/controls/wizard.rs](storage-app/src/controls/wizard.rs)
- [x] Temporary compatibility wrapper retired
  - Evidence: [storage-app/src/ui/mod.rs](storage-app/src/ui/mod.rs) no longer declares `wizard`
- [x] Move row/info/link helpers into controls fields
  - Evidence: [storage-app/src/controls/fields.rs](storage-app/src/controls/fields.rs)
- [x] Move callout/status helpers into controls status
  - Evidence: [storage-app/src/controls/status.rs](storage-app/src/controls/status.rs)
- [x] Update call sites to controls imports
  - Evidence examples:
    - [storage-app/src/views/app.rs](storage-app/src/views/app.rs)
    - [storage-app/src/views/network.rs](storage-app/src/views/network.rs)

Status: **Complete**

---

### Task 3 — Migrate app/network/dialog rendering into views

- [x] Migrate app rendering to canonical views module
  - Evidence: [storage-app/src/views/app.rs](storage-app/src/views/app.rs)
- [x] Migrate network rendering to canonical views module
  - Evidence: [storage-app/src/views/network.rs](storage-app/src/views/network.rs)
- [x] Split dialogs rendering into planned canonical modules
  - Evidence:
    - [storage-app/src/views/dialogs/mod.rs](storage-app/src/views/dialogs/mod.rs)
    - [storage-app/src/views/dialogs/disk.rs](storage-app/src/views/dialogs/disk.rs)
    - [storage-app/src/views/dialogs/encryption.rs](storage-app/src/views/dialogs/encryption.rs)
    - [storage-app/src/views/dialogs/mount.rs](storage-app/src/views/dialogs/mount.rs)
    - [storage-app/src/views/dialogs/partition.rs](storage-app/src/views/dialogs/partition.rs)
    - [storage-app/src/views/dialogs/image.rs](storage-app/src/views/dialogs/image.rs)
    - [storage-app/src/views/dialogs/btrfs.rs](storage-app/src/views/dialogs/btrfs.rs)
    - [storage-app/src/views/dialogs/common.rs](storage-app/src/views/dialogs/common.rs)
- [x] Remove dependency on legacy dialog view implementation
  - Evidence: [storage-app/src/ui/dialogs/mod.rs](storage-app/src/ui/dialogs/mod.rs) no longer exposes `view` and `storage-app/src/ui/dialogs/view` removed

Status: **Complete**

---

### Task 4 — Migrate volumes/sidebar/btrfs rendering

- [x] Move volumes rendering to canonical views module
  - Evidence: [storage-app/src/views/volumes.rs](storage-app/src/views/volumes.rs)
- [x] Move sidebar rendering to canonical views module
  - Evidence: [storage-app/src/views/sidebar.rs](storage-app/src/views/sidebar.rs)
- [x] Move btrfs rendering to canonical views module
  - Evidence: [storage-app/src/views/btrfs.rs](storage-app/src/views/btrfs.rs)
- [x] Remove legacy wrappers for these domains
  - Evidence: no remaining files under `storage-app/src/ui/{volumes,sidebar,btrfs,network}/view.rs`
- [x] Replace repeated strips/rows with controls where extraction was straightforward
  - Evidence examples:
    - [storage-app/src/views/network.rs](storage-app/src/views/network.rs) now uses controls actions/form/layout
    - [storage-app/src/views/sidebar.rs](storage-app/src/views/sidebar.rs) now uses controls layout

Status: **Complete**

---

### Task 5 — Eliminate visual utils bucket

- [x] Remove visual utils module
  - Evidence: [storage-app/src/utils/ui.rs](storage-app/src/utils/ui.rs) deleted
- [x] Remove visual utils re-exports
  - Evidence: [storage-app/src/utils/mod.rs](storage-app/src/utils/mod.rs)
- [x] Keep only non-visual utilities in utils
  - Evidence:
    - [storage-app/src/utils/segments.rs](storage-app/src/utils/segments.rs)
    - [storage-app/src/utils/unit_size_input.rs](storage-app/src/utils/unit_size_input.rs)

Status: **Complete**

---

### Task 6 — Remove legacy wrappers and finalize structure

- [x] Remove `ui/*/view.rs` wrappers for app/network/volumes/sidebar/btrfs
  - Evidence: no remaining files matching `storage-app/src/ui/**/view.rs`
- [x] Remove legacy dialogs view layer
  - Evidence: [storage-app/src/ui/dialogs/mod.rs](storage-app/src/ui/dialogs/mod.rs)
- [x] Point dialogs rendering directly to canonical views/dialogs modules
  - Evidence: [storage-app/src/views/dialogs/mod.rs](storage-app/src/views/dialogs/mod.rs)
- [x] Remove temporary wizard shim
  - Evidence: [storage-app/src/ui/mod.rs](storage-app/src/ui/mod.rs) and deletion of `storage-app/src/ui/wizard.rs`

Status: **Complete**

## Component Move Ledger (Exact)

### Created and now canonical

- [storage-app/src/views/app.rs](storage-app/src/views/app.rs) (full implementation)
- [storage-app/src/views/network.rs](storage-app/src/views/network.rs) (full implementation)
- [storage-app/src/views/volumes.rs](storage-app/src/views/volumes.rs) (full implementation)
- [storage-app/src/views/sidebar.rs](storage-app/src/views/sidebar.rs) (full implementation)
- [storage-app/src/views/btrfs.rs](storage-app/src/views/btrfs.rs) (full implementation)
- [storage-app/src/views/dialogs/mod.rs](storage-app/src/views/dialogs/mod.rs) (module exports)
- [storage-app/src/views/dialogs/disk.rs](storage-app/src/views/dialogs/disk.rs) (full implementation)
- [storage-app/src/views/dialogs/encryption.rs](storage-app/src/views/dialogs/encryption.rs) (full implementation)
- [storage-app/src/views/dialogs/mount.rs](storage-app/src/views/dialogs/mount.rs) (full implementation)
- [storage-app/src/views/dialogs/partition.rs](storage-app/src/views/dialogs/partition.rs) (full implementation)
- [storage-app/src/views/dialogs/image.rs](storage-app/src/views/dialogs/image.rs) (full implementation)
- [storage-app/src/views/dialogs/btrfs.rs](storage-app/src/views/dialogs/btrfs.rs) (full implementation)
- [storage-app/src/views/dialogs/common.rs](storage-app/src/views/dialogs/common.rs) (full implementation)

### Removed wrappers

- [storage-app/src/ui/app/view.rs](storage-app/src/ui/app/view.rs) deleted
- [storage-app/src/ui/network/view.rs](storage-app/src/ui/network/view.rs) deleted
- [storage-app/src/ui/volumes/view.rs](storage-app/src/ui/volumes/view.rs) deleted
- [storage-app/src/ui/sidebar/view.rs](storage-app/src/ui/sidebar/view.rs) deleted
- [storage-app/src/ui/btrfs/view.rs](storage-app/src/ui/btrfs/view.rs) deleted
- [storage-app/src/ui/dialogs/view](storage-app/src/ui/dialogs/view) directory deleted
- [storage-app/src/ui/wizard.rs](storage-app/src/ui/wizard.rs) deleted
- [storage-app/src/utils/ui.rs](storage-app/src/utils/ui.rs) deleted

## Next Optional Improvements

1. Extract additional repeated dialog/form micro-patterns into [storage-app/src/controls/form.rs](storage-app/src/controls/form.rs) and [storage-app/src/controls/layout.rs](storage-app/src/controls/layout.rs).
2. Add focused snapshot tests or UI integration checks for multi-step dialogs if desired.
