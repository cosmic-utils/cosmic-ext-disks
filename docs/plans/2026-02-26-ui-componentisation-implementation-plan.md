# UI Componentisation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Introduce a reusable `controls/` layer, consolidate domain rendering in `views/`, and remove visual composition from `utils/` while keeping behavior unchanged.

**Architecture:** Migrate incrementally: first create controls and move existing reusable primitives, then migrate domain view modules one surface at a time, then remove legacy wrappers. Keep state/message/update modules in `ui/*` and make `views/*` the canonical rendering entrypoints that compose `controls/*`.

**Tech Stack:** Rust, libcosmic/iced, workspace lint/test verification (`cargo fmt`, `cargo clippy`, `cargo test --no-run`).

---

### Task 1: Scaffold controls layer

**Files:**
- Create: `storage-app/src/controls/mod.rs`
- Create: `storage-app/src/controls/layout.rs`
- Create: `storage-app/src/controls/fields.rs`
- Create: `storage-app/src/controls/actions.rs`
- Create: `storage-app/src/controls/status.rs`
- Create: `storage-app/src/controls/form.rs`
- Create: `storage-app/src/controls/wizard.rs`
- Modify: `storage-app/src/main.rs` (module wiring)

**Steps:**
1. Add empty module files and exports.
2. Wire new `controls` module in crate root.
3. Run `cargo check -p cosmic-ext-storage`.
4. Commit scaffold.

### Task 2: Move existing reusable helpers to controls

**Files:**
- Modify: `storage-app/src/ui/wizard.rs`
- Modify: `storage-app/src/utils/ui.rs`
- Modify: `storage-app/src/utils/mod.rs`
- Modify: call sites under `storage-app/src/ui/**`
- Modify/Create: `storage-app/src/controls/{wizard,fields,status}.rs`

**Steps:**
1. Move wizard primitives into `controls/wizard.rs`; keep public signatures stable.
2. Move row/info/link helpers into `controls/fields.rs`.
3. Move callout/status helpers into `controls/status.rs`.
4. Update imports at all call sites.
5. Keep temporary forwarding wrappers only if needed for one-pass migration.
6. Run `cargo clippy --workspace --all-targets`.
7. Commit.

### Task 3: Migrate app/network/dialog rendering into views

**Files:**
- Create: `storage-app/src/views/app.rs`
- Create: `storage-app/src/views/network.rs`
- Create: `storage-app/src/views/dialogs/mod.rs`
- Create: `storage-app/src/views/dialogs/{disk,encryption,mount,partition,image,btrfs,common}.rs`
- Modify: `storage-app/src/views/mod.rs`
- Modify: `storage-app/src/ui/app/view.rs`
- Modify: `storage-app/src/ui/network/view.rs`
- Modify: `storage-app/src/ui/dialogs/view/*.rs`

**Steps:**
1. Move rendering functions into `views/*` modules.
2. Update `ui/*/view.rs` to either re-export or thin-forward temporarily.
3. Replace repeated ad hoc rows/cards with controls primitives where extraction is straightforward.
4. Run `cargo check -p cosmic-ext-storage`.
5. Commit.

### Task 4: Migrate volumes/sidebar/btrfs rendering

**Files:**
- Create: `storage-app/src/views/volumes.rs`
- Create: `storage-app/src/views/sidebar.rs`
- Create: `storage-app/src/views/btrfs.rs`
- Modify: `storage-app/src/ui/volumes/view.rs`
- Modify: `storage-app/src/ui/volumes/disk_header.rs`
- Modify: `storage-app/src/ui/sidebar/view.rs`
- Modify: `storage-app/src/ui/btrfs/view.rs`

**Steps:**
1. Move render logic into `views/*` files.
2. Replace repeated info rows/action strips with `controls/fields` and `controls/actions`.
3. Keep behavior and message wiring unchanged.
4. Run `cargo clippy --workspace --all-targets`.
5. Commit.

### Task 5: Eliminate visual utils bucket

**Files:**
- Modify: `storage-app/src/utils/unit_size_input.rs`
- Modify: `storage-app/src/utils/segments.rs`
- Modify: `storage-app/src/utils/mod.rs`
- Delete: `storage-app/src/utils/ui.rs`
- Optional Delete/Refactor: `storage-app/src/utils/mod.rs` (if empty/unneeded)

**Steps:**
1. Move remaining visual composition logic from `utils` into `controls`.
2. Keep non-visual utility logic in domain-appropriate modules.
3. Remove `utils/ui.rs` and fix imports.
4. Run `cargo check -p cosmic-ext-storage`.
5. Commit.

### Task 6: Remove legacy wrappers and finalize structure

**Files:**
- Delete or reduce: `storage-app/src/ui/*/view.rs` legacy wrappers
- Modify: `storage-app/src/ui/mod.rs`
- Modify: all imports pointing to legacy view wrappers

**Steps:**
1. Remove temporary compatibility re-exports.
2. Point all rendering imports to canonical `views/*` modules.
3. Run full verification:
   - `cargo fmt --all`
   - `cargo clippy --workspace --all-targets`
   - `cargo test --workspace --no-run`
4. Commit final cleanup.
