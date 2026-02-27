# Async Sidebar Loading Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make startup load sidebar sections concurrently, stream drives into the sidebar as each drive finishes, and show per-section loading spinners with per-drive timing logs.

**Architecture:** Startup dispatch moves from chained tasks to parallel tasks. Drive loading is converted to incremental message-driven updates (start/item/finish). Sidebar state tracks section loading flags so headers render loading indicators while data is in flight.

**Tech Stack:** Rust, COSMIC application/task runtime, `storage_contracts` DBus clients, existing sidebar/nav state model.

---

### Task 1: Add incremental startup message types

**Files:**
- Modify: `storage-app/src/message/app.rs`

**Step 1: Write failing compile target (message usage first)**
- Add references in later files first (next task) expecting new messages to exist.

**Step 2: Run compile to confirm missing variants**
Run: `cargo check -p storage-app`
Expected: FAIL with missing `Message` variants.

**Step 3: Add minimal message variants**
- Add startup drive streaming messages:
  - `DriveLoadStarted { total: usize }`
  - `DriveLoaded { result: Result<crate::models::UiDrive, String>, elapsed_ms: u128 }`
  - `DriveLoadFinished`
- Add optional section-state messages only if needed for clean update isolation.

**Step 4: Re-run compile**
Run: `cargo check -p storage-app`
Expected: progresses to next missing-handler errors in update/app flow.

**Step 5: Commit**
```bash
git add storage-app/src/message/app.rs
git commit -m "feat(app): add incremental drive loading messages"
```

### Task 2: Extend sidebar state for loading indicators

**Files:**
- Modify: `storage-app/src/state/sidebar.rs`
- Test: `storage-app/src/state/sidebar.rs` (unit tests module)

**Step 1: Write failing state tests**
- Add tests for loading flags transitions:
  - defaults are false,
  - drive loading start sets drive-related sections loading,
  - finish clears drive-related loading,
  - network/logical can toggle independently.

**Step 2: Run targeted tests and confirm fail**
Run: `cargo test -p storage-app state::sidebar -- --nocapture`
Expected: FAIL until new fields/helpers are implemented.

**Step 3: Add minimal state fields/helpers**
- Add a compact struct or fields on `SidebarState` for section loading booleans.
- Add helper methods to set/clear loading states with simple deterministic behavior.

**Step 4: Re-run tests**
Run: `cargo test -p storage-app state::sidebar -- --nocapture`
Expected: PASS for new sidebar loading tests.

**Step 5: Commit**
```bash
git add storage-app/src/state/sidebar.rs
git commit -m "feat(sidebar): track section loading state"
```

### Task 3: Implement incremental drive loader with timing

**Files:**
- Modify: `storage-app/src/models/load.rs`

**Step 1: Write failing loader test (ordering helper)
**
- Add a pure helper test around deterministic ordering/merge behavior if loader internals are hard to unit-test directly.

**Step 2: Run targeted tests and confirm fail**
Run: `cargo test -p storage-app models::load -- --nocapture`
Expected: FAIL for missing helper/behavior.

**Step 3: Add minimal incremental loader API**
- Keep existing `load_all_drives()` for full refresh callers.
- Add new async path used by startup to:
  - list disks once,
  - build each `UiDrive` in its own task,
  - return per-drive completion events with elapsed timing.
- Add `tracing::info!` per drive build with elapsed ms and device.

**Step 4: Re-run tests**
Run: `cargo test -p storage-app models::load -- --nocapture`
Expected: PASS for added helper tests.

**Step 5: Commit**
```bash
git add storage-app/src/models/load.rs
git commit -m "feat(load): add incremental drive loading with timing logs"
```

### Task 4: Dispatch startup tasks in parallel

**Files:**
- Modify: `storage-app/src/app.rs`

**Step 1: Write compile-failing wiring in `init`**
- Replace chained startup command flow with `Task::batch` and wire new drive messages.

**Step 2: Run compile to validate wiring gaps**
Run: `cargo check -p storage-app`
Expected: FAIL in update handlers until message handling is added.

**Step 3: Implement minimal init orchestration changes**
- Emit `DriveLoadStarted` before spawning drive tasks.
- Emit per-drive `DriveLoaded` messages as tasks complete.
- Emit `DriveLoadFinished` after completion.
- Keep tools/network/logical startup tasks independent and concurrent.

**Step 4: Re-run compile**
Run: `cargo check -p storage-app`
Expected: PASS or only remaining update/view errors.

**Step 5: Commit**
```bash
git add storage-app/src/app.rs
git commit -m "feat(app): run startup loaders in parallel"
```

### Task 5: Handle incremental updates in update flow

**Files:**
- Modify: `storage-app/src/update/mod.rs`
- Modify: `storage-app/src/update/nav.rs`

**Step 1: Add failing tests for merge/order helper (if feasible)
**
- Prefer a pure helper in `update/nav.rs` for insertion + stable ordering to enable unit tests.

**Step 2: Run targeted tests and confirm fail**
Run: `cargo test -p storage-app update::nav -- --nocapture`
Expected: FAIL until helper + handlers are implemented.

**Step 3: Add minimal update handlers**
- `DriveLoadStarted`: set section loading states true and clear startup drive cache.
- `DriveLoaded`: on success, merge drive into in-memory list with stable deterministic ordering; rebuild nav/sidebar preserving selection.
- `DriveLoadFinished`: clear drive/images loading flags.
- Ensure network/logical completion handlers clear their own loading flags.

**Step 4: Re-run tests**
Run: `cargo test -p storage-app update::nav -- --nocapture`
Expected: PASS for new helper tests.

**Step 5: Commit**
```bash
git add storage-app/src/update/mod.rs storage-app/src/update/nav.rs
git commit -m "feat(update): support incremental sidebar/nav drive updates"
```

### Task 6: Add section header spinner UI

**Files:**
- Modify: `storage-app/src/views/sidebar.rs`
- Modify: `storage-app/src/views/network.rs` (if shared header helper is reused)

**Step 1: Write failing view-level test or snapshot check (if project pattern exists)**
- If no view tests exist, skip test creation and proceed with compile+manual verification.

**Step 2: Implement minimal spinner integration**
- Update section header renderers to show spinner to the right of header text while section loading flag is true.
- Preserve existing controls/add buttons (Images/Network).
- Use existing COSMIC widget/theme primitives only.

**Step 3: Compile check**
Run: `cargo check -p storage-app`
Expected: PASS.

**Step 4: Manual verification run**
Run: `RUST_LOG=cosmic_ext_storage=info just app`
Expected:
- section header spinners visible during loading,
- drives pop in incrementally,
- per-drive timing logs at info level,
- network/logical spinner behavior independent of drive completion.

**Step 5: Commit**
```bash
git add storage-app/src/views/sidebar.rs storage-app/src/views/network.rs
git commit -m "feat(sidebar): add section loading spinners"
```

### Task 7: Regression checks and docs touch-up

**Files:**
- Modify: `storage-app/README.md` (only if startup behavior/logging docs need note)

**Step 1: Run focused verification**
Run: `cargo test -p storage-app`
Expected: PASS for storage-app tests.

**Step 2: Run workspace sanity check**
Run: `cargo check`
Expected: PASS.

**Step 3: Validate no UX regressions in selection behavior**
- Manual checks:
  - selected drive remains stable while new drives arrive,
  - selected child persists where expected,
  - network and logical sections remain interactive once loaded.

**Step 4: Optional docs update**
- Add short note about incremental startup and performance logs if needed.

**Step 5: Commit**
```bash
git add storage-app/README.md
git commit -m "docs(app): note incremental sidebar loading behavior"
```
