# Usage Scan Single-Page Wizard Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Show a single-page wizard before each Usage scan, allowing mount-point selection, show-all control, and parallelism preset selection before scan launch.

**Architecture:** Extend existing Usage tab state/message/update flow with a wizard substate that intercepts refresh/start scan. Keep wizard in the Usage tab view and pass selected mount roots + controls through existing client/service scan pipeline.

**Tech Stack:** Rust workspace (`storage-ui`, `storage-service`, `storage-sys`, `storage-common`), COSMIC/iced widgets, zbus D-Bus API, serde.

---

### Task 1: Add shared scan options model for selected mounts

**Files:**
- Modify: `storage-common/src/usage_scan.rs`
- Modify: `storage-common/src/lib.rs`
- Test: `storage-common/src/usage_scan.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn usage_scan_request_roundtrip_with_mount_roots() {
    let request = UsageScanRequest {
        scan_id: "scan-1".into(),
        top_files_per_category: 20,
        show_all_files: false,
        parallelism_preset: UsageScanParallelismPreset::Balanced,
        mount_points: vec!["/".into(), "/home".into()],
    };

    let json = serde_json::to_string(&request).expect("serialize request");
    let parsed: UsageScanRequest = serde_json::from_str(&json).expect("parse request");
    assert_eq!(parsed.mount_points, vec!["/", "/home"]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-common usage_scan_request_roundtrip_with_mount_roots -v`  
Expected: FAIL because `mount_points` does not exist.

**Step 3: Write minimal implementation**

- Add `mount_points: Vec<String>` to shared usage scan request model.
- Keep existing serde derives.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-common usage_scan_request_roundtrip_with_mount_roots -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-common/src/usage_scan.rs storage-common/src/lib.rs
git commit -m "feat(storage-common): add usage scan mount roots to request model"
```

---

### Task 2: Extend service scan API for selected mount roots

**Files:**
- Modify: `storage-service/src/filesystems.rs`
- Test: `storage-service/src/filesystems.rs` (module tests)

**Step 1: Write the failing test**

```rust
#[test]
fn map_parallelism_threads_uses_expected_ratios() {
    assert_eq!(map_parallelism_threads(UsageScanParallelismPreset::Low, 8), 2);
    assert_eq!(map_parallelism_threads(UsageScanParallelismPreset::Balanced, 8), 4);
    assert_eq!(map_parallelism_threads(UsageScanParallelismPreset::High, 8), 8);
}
```

**Step 2: Run test to verify baseline behavior**

Run: `cargo test -p storage-service map_parallelism_threads_uses_expected_ratios -v`  
Expected: PASS/FAIL depending on current helper availability; stabilize before API wiring.

**Step 3: Write minimal implementation**

- Extend `get_usage_scan` D-Bus method signature to accept selected mount points.
- Validate mount list (must be non-empty and absolute paths).
- Build scan roots from selected mounts instead of always discovering all local mounts.
- Keep existing show-all auth and parallelism mapping logic.

**Step 4: Run checks**

Run: `cargo check -p storage-service`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-service/src/filesystems.rs
git commit -m "feat(storage-service): scan selected mount roots for usage wizard"
```

---

### Task 3: Extend UI client proxy scan signature

**Files:**
- Modify: `storage-ui/src/client/filesystems.rs`

**Step 1: Write the failing compile scenario**

- Update caller site signature in a temporary branch of code to include mount roots (compile should fail until proxy is updated).

**Step 2: Run check to verify failure**

Run: `cargo check -p cosmic-ext-storage`  
Expected: FAIL with `get_usage_scan` argument mismatch.

**Step 3: Write minimal implementation**

- Add `mount_points_json` (or structured mount list argument) to proxy method and typed client method.
- Serialize mount selections in client layer.

**Step 4: Run check to verify pass**

Run: `cargo check -p cosmic-ext-storage`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-ui/src/client/filesystems.rs
git commit -m "feat(storage-ui): pass usage wizard mount roots in scan request"
```

---

### Task 4: Add Usage wizard state + messages

**Files:**
- Modify: `storage-ui/src/ui/volumes/state.rs`
- Modify: `storage-ui/src/ui/app/message.rs`
- Modify: `storage-ui/src/ui/app/update/mod.rs`

**Step 1: Write failing reducer test**

```rust
#[test]
fn usage_refresh_opens_wizard_instead_of_starting_scan() {
    // Given Usage tab active
    // When Message::UsageRefreshRequested
    // Then wizard_open == true and no scan task dispatched
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage usage_refresh_opens_wizard_instead_of_starting_scan -v`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- Add wizard state fields to `UsageTabState`.
- Add messages for:
  - open/close wizard,
  - mount toggle,
  - wizard show-all toggle,
  - wizard parallelism selection,
  - wizard start scan.
- Update reducer so `UsageRefreshRequested` opens wizard.

**Step 4: Run checks**

Run: `cargo check -p cosmic-ext-storage`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-ui/src/ui/volumes/state.rs storage-ui/src/ui/app/message.rs storage-ui/src/ui/app/update/mod.rs
git commit -m "feat(storage-ui): add usage pre-scan wizard state and reducer flow"
```

---

### Task 5: Render single-page wizard in Usage view

**Files:**
- Modify: `storage-ui/src/ui/app/view.rs`

**Step 1: Write failing view test (or compile-level assertion)**

```rust
#[test]
fn usage_wizard_renders_mounts_show_all_parallelism_and_actions() {
    // Build minimal state with wizard_open = true
    // Assert expected control labels are present in rendered tree/snapshot.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage usage_wizard_renders_mounts_show_all_parallelism_and_actions -v`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- Add single-page wizard layout in Usage tab.
- Render mount-point multi-select checkboxes.
- Render `Show All Files` and parallelism dropdown.
- Add footer `Cancel` and `Start Scan` buttons.
- Keep existing post-start scan view unchanged.

**Step 4: Run check to verify pass**

Run: `cargo check -p cosmic-ext-storage`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-ui/src/ui/app/view.rs
git commit -m "feat(storage-ui): render usage pre-scan single-page wizard"
```

---

### Task 6: Wire wizard start to actual scan launch

**Files:**
- Modify: `storage-ui/src/ui/app/update/mod.rs`

**Step 1: Write failing reducer test**

```rust
#[test]
fn usage_wizard_start_dispatches_scan_with_selected_mounts() {
    // set wizard selections
    // send start message
    // assert scan load payload includes selected mounts/show_all/parallelism
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage usage_wizard_start_dispatches_scan_with_selected_mounts -v`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- On wizard start, validate at least one mount selected.
- Close wizard and dispatch scan load with selected values.
- Keep wizard open with inline error if validation/auth fails.

**Step 4: Run checks**

Run: `cargo check -p cosmic-ext-storage -p storage-service`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-ui/src/ui/app/update/mod.rs
git commit -m "feat(storage-ui): launch usage scans from wizard selections"
```

---

### Task 7: Final verification

**Step 1: Run targeted checks**

Run:
- `cargo check -p storage-common -p storage-service -p cosmic-ext-storage`
- `cargo test -p storage-common -v`

Expected: PASS.

**Step 2: Manual smoke checks**

Verify:
- Refresh always opens wizard.
- Mount subset selection impacts indexed results.
- Show-all and parallelism controls are respected on scan start.
- Cancel preserves current results.

**Step 3: Commit any doc touchups**

```bash
git add README.md
git commit -m "docs: describe usage pre-scan wizard behavior"
```

---

## Notes
- Keep this as a single-page wizard only.
- Use rclone wizard style cues, not its multi-step flow.
- Avoid adding extra routes/modals unless requirements change.
