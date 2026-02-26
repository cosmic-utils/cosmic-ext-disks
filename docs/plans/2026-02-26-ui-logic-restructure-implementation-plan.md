# UI Logic Restructure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move feature logic from `ui/*` to `message/state/update` layer-first modules, remove `ui` architectural usage, and finish deduplicating action/styling primitives.

**Architecture:** Use a direct, no-compat migration. Physically move files into `message/*`, `state/*`, `update/*`, and feature helper roots, then rewrite imports globally and delete `ui/*` leftovers. Keep behavior unchanged while reducing duplication via `controls/actions` and `controls/layout`.

**Tech Stack:** Rust, libcosmic/iced, cargo workspace verification (`cargo clippy`, `cargo test --no-run`).

---

### Task 1: Create destination module skeletons

**Files:**
- Create: `storage-app/src/message/mod.rs`
- Create: `storage-app/src/state/mod.rs`
- Create: `storage-app/src/update/mod.rs`
- Create: `storage-app/src/update/volumes/mod.rs`
- Modify: `storage-app/src/main.rs` (module wiring)

**Steps:**
1. Create destination module directories/files.
2. Export canonical module names from each `mod.rs`.
3. Wire `mod message; mod state; mod update;` in `main.rs`.
4. Run: `cargo check -p cosmic-ext-storage`.
5. Commit: `refactor(storage-app): scaffold message state update layers`.

### Task 2: Migrate message modules

**Files:**
- Move: `storage-app/src/ui/app/message.rs` -> `storage-app/src/message/app.rs`
- Move: `storage-app/src/ui/dialogs/message.rs` -> `storage-app/src/message/dialogs.rs`
- Move: `storage-app/src/ui/network/message.rs` -> `storage-app/src/message/network.rs`
- Move: `storage-app/src/ui/volumes/message.rs` -> `storage-app/src/message/volumes.rs`
- Modify: call sites importing `crate::ui::*::message`

**Steps:**
1. Move files physically to new locations.
2. Rewrite imports to `crate::message::*`.
3. Fix cross-message references (e.g., dialogs step enums pointing to `state::*`).
4. Run: `cargo clippy --workspace --all-targets`.
5. Commit: `refactor(storage-app): move ui messages into message layer`.

### Task 3: Migrate state modules

**Files:**
- Move: `storage-app/src/ui/app/state.rs` -> `storage-app/src/state/app.rs`
- Move: `storage-app/src/ui/btrfs/state.rs` -> `storage-app/src/state/btrfs.rs`
- Move: `storage-app/src/ui/dialogs/state.rs` -> `storage-app/src/state/dialogs.rs`
- Move: `storage-app/src/ui/network/state.rs` -> `storage-app/src/state/network.rs`
- Move: `storage-app/src/ui/sidebar/state.rs` -> `storage-app/src/state/sidebar.rs`
- Move: `storage-app/src/ui/volumes/state.rs` -> `storage-app/src/state/volumes.rs`
- Modify: call sites importing `crate::ui::*::state`

**Steps:**
1. Move files physically.
2. Rewrite all `state` imports to `crate::state::*`.
3. Fix enum/type path references from message/views/update.
4. Run: `cargo clippy --workspace --all-targets`.
5. Commit: `refactor(storage-app): move ui state into state layer`.

### Task 4: Migrate app update graph

**Files:**
- Move: `storage-app/src/ui/app/update/mod.rs` -> `storage-app/src/updates/mod.rs`
- Move: `storage-app/src/ui/app/update/mod.rs` -> `storage-app/src/update/mod.rs`
- Move: `storage-app/src/ui/app/update/{btrfs,drive,image,nav,network,smart}.rs` -> `storage-app/src/update/`
- Move: `storage-app/src/ui/app/update/image/{dialogs,ops}.rs` -> `storage-app/src/update/image/`
- Modify: imports from `super::message/state` to `crate::message::app` and `crate::state::app`

**Steps:**
1. Move update files preserving tree.
2. Fix module declarations and import paths.
3. Rewire app entrypoint to call `crate::update::*`.
4. Run: `cargo clippy --workspace --all-targets`.
5. Commit: `refactor(storage-app): migrate app update graph to updates layer`.

### Task 5: Migrate volumes update graph

**Files:**
- Move: `storage-app/src/ui/volumes/update.rs` -> `storage-app/src/updates/volumes/mod.rs`
- Move: `storage-app/src/ui/volumes/update.rs` -> `storage-app/src/update/volumes/mod.rs`
- Move: `storage-app/src/ui/volumes/update/{btrfs,create,encryption,filesystem,mount,mount_options,partition,selection}.rs` -> `storage-app/src/update/volumes/`
- Modify: imports for dialogs/message/state paths

**Steps:**
1. Move root and child update files.
2. Fix module declarations and imports.
3. Rewire `VolumesControl::update` call sites to `crate::update::volumes`.
4. Run: `cargo clippy --workspace --all-targets`.
5. Commit: `refactor(storage-app): migrate volumes update graph to updates layer`.

### Task 6: Move remaining ui helper modules out of ui

**Files:**
- Move: `storage-app/src/ui/network/icons.rs` -> `storage-app/src/network/icons.rs`
- Move: `storage-app/src/ui/volumes/helpers.rs` -> `storage-app/src/volumes/helpers.rs`
- Move: `storage-app/src/ui/volumes/disk_header.rs` -> `storage-app/src/volumes/disk_header.rs`
- Move: `storage-app/src/ui/volumes/usage_pie.rs` -> `storage-app/src/volumes/usage_pie.rs`
- Move: `storage-app/src/ui/app/subscriptions.rs` -> `storage-app/src/subscriptions/app.rs`
- Move: `storage-app/src/ui/error.rs` -> `storage-app/src/errors/ui.rs`
- Modify: all imports/callers

**Steps:**
1. Move helper files to non-ui destinations.
2. Re-export from new roots if necessary (`network`, `volumes`, `subscriptions`, `errors`).
3. Replace all `crate::ui::*` helper imports.
4. Run: `cargo clippy --workspace --all-targets`.
5. Commit: `refactor(storage-app): move remaining ui helpers to feature roots`.

### Task 7: Remove ui module tree and dead module glue

**Files:**
- Modify/Delete: `storage-app/src/ui/mod.rs` and feature `ui/*/mod.rs` files
- Modify: `storage-app/src/app.rs` (re-export from `message::app` and `state::app`)
- Modify: any remaining module wiring across `views/*`, `app.rs`, `update/*`

**Steps:**
1. Remove obsolete `ui/*` module declarations.
2. Delete empty/unreferenced `ui` directories/files.
3. Ensure no `crate::ui::` imports remain.
4. Run: `cargo clippy --workspace --all-targets`.
5. Commit: `refactor(storage-app): remove ui layer after logic migration`.

### Task 8: Finish action/styling deduplication pass

**Files:**
- Modify: `storage-app/src/controls/actions.rs`
- Modify: `storage-app/src/controls/layout.rs`
- Modify: `storage-app/src/views/{app,network,sidebar,volumes}.rs`

**Steps:**
1. Identify repeated action button/styling blocks in views.
2. Extract common builders/styles into controls.
3. Replace repeated inline style/button builders in views.
4. Run: `cargo clippy --workspace --all-targets`.
5. Commit: `refactor(storage-app): deduplicate view action and style primitives`.

### Task 9: Final verification and cleanup

**Files:**
- Modify: docs/plan files if structure changed during implementation

**Steps:**
1. Run: `cargo fmt --all`.
2. Run: `cargo clippy --workspace --all-targets`.
3. Run: `cargo test --workspace --no-run`.
4. Confirm clean working tree.
5. Commit: `chore(storage-app): finalize ui logic restructure verification`.
