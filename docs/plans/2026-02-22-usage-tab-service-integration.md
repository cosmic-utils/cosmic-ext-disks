# Usage Tab Service Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expose global usage scan via service and render it in a new UI Usage tab with bar chart, category tabs, file grid, and loading progress.

**Architecture:** Define canonical usage DTOs in `storage-common`, make `storage-sys` produce those DTOs directly, pass through service/dbus/client layers, and bind them to a new `Usage` tab state/view in `storage-ui`.

**Tech Stack:** Rust 2021, zbus D-Bus methods/signals, existing cosmic/iced UI components, serde JSON DTO serialization.

---

### Task 1: Introduce canonical usage DTOs in `storage-common`

**Files:**
- Create: `storage-common/src/usage_scan.rs`
- Modify: `storage-common/src/lib.rs`
- Test: `storage-common/src/usage_scan.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn usage_scan_result_roundtrip_serialization() {
    // construct UsageScanResult with one segment and one top-file entry
    // serialize + deserialize and assert equality
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-common usage_scan_result_roundtrip_serialization -v`  
Expected: FAIL due to missing module/types.

**Step 3: Write minimal implementation**

Define DTOs:
- `UsageCategory`
- `UsageSegment`
- `UsageTopFileEntry`
- `UsageCategoryFiles`
- `UsageScanProgress`
- `UsageScanResult`

Derive serde traits and keep fields minimal for current requirements.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-common usage_scan_result_roundtrip_serialization -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-common/src/usage_scan.rs storage-common/src/lib.rs
git commit -m "feat(storage-common): add canonical usage scan DTOs"
```

---

### Task 2: Make `storage-sys` emit canonical DTOs directly (no mirror DTOs)

**Files:**
- Modify: `storage-sys/src/usage/types.rs`
- Modify: `storage-sys/src/usage/scanner.rs`
- Modify: `storage-sys/src/usage/mod.rs`
- Test: `storage-sys/src/usage/scanner.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn scanner_outputs_storage_common_usage_scan_result() {
    // call scanner
    // type-check and verify returned DTO is storage_common::usage_scan::UsageScanResult
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys scanner_outputs_storage_common_usage_scan_result -v`  
Expected: FAIL due to local mirror type usage.

**Step 3: Write minimal implementation**

- Replace or alias local usage result structs in sys to canonical `storage-common` DTOs.
- Keep scanner internal helpers private but map directly into canonical output.
- Preserve existing top-file and category ordering behavior.

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys scanner_outputs_storage_common_usage_scan_result -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/types.rs storage-sys/src/usage/scanner.rs storage-sys/src/usage/mod.rs
git commit -m "feat(storage-sys): emit canonical usage scan DTOs directly"
```

---

### Task 3: Expose global usage scan in service and dbus

**Files:**
- Modify: `storage-service/src/service.rs` (or appropriate manager interface file)
- Modify: `storage-service/src/main.rs` (if interface wiring needed)
- Modify: `storage-dbus/src/lib.rs`
- Create/Modify: `storage-dbus/src/usage_scan.rs` (if split module preferred)
- Test: service/dbus serialization tests where available

**Step 1: Write the failing test**

```rust
#[test]
fn dbus_usage_scan_result_serializes_with_canonical_schema() {
    // simulate service response payload
    // assert expected schema fields exist
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-dbus dbus_usage_scan_result_serializes_with_canonical_schema -v`  
Expected: FAIL due to missing API.

**Step 3: Write minimal implementation**

- Add global usage scan method in service D-Bus interface.
- Return canonical DTO payload (json string or typed payload as existing pattern dictates).
- Add progress response support for loading bar needs (poll method or signal payload).

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-dbus dbus_usage_scan_result_serializes_with_canonical_schema -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-service/src/service.rs storage-service/src/main.rs storage-dbus/src/lib.rs storage-dbus/src/usage_scan.rs
git commit -m "feat(service): expose global usage scan API with progress"
```

---

### Task 4: Add UI client API for usage scan + progress

**Files:**
- Create/Modify: `storage-ui/src/client/usage.rs` (or adjacent client module)
- Modify: `storage-ui/src/client/mod.rs`
- Test: client parsing tests

**Step 1: Write the failing test**

```rust
#[test]
fn usage_client_parses_scan_result_and_progress() {
    // feed sample JSON payloads
    // assert canonical DTO parse success
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-ui usage_client_parses_scan_result_and_progress -v`  
Expected: FAIL due to missing client methods.

**Step 3: Write minimal implementation**

- Add client methods for scan request and progress retrieval/subscription.
- Deserialize into canonical DTOs from `storage-common`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-ui usage_client_parses_scan_result_and_progress -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-ui/src/client/usage.rs storage-ui/src/client/mod.rs
git commit -m "feat(storage-ui): add usage scan client API"
```

---

### Task 5: Add new `Usage` tab state and messages in UI control flow

**Files:**
- Modify: `storage-ui/src/ui/volumes/state.rs`
- Modify: `storage-ui/src/ui/volumes/message.rs` (or equivalent message enums)
- Modify: `storage-ui/src/ui/volumes/update.rs`
- Modify: `storage-ui/src/ui/volumes/mod.rs`
- Test: state transition tests where present

**Step 1: Write the failing test**

```rust
#[test]
fn usage_tab_transitions_loading_progress_loaded() {
    // switch to Usage tab
    // apply progress message
    // apply loaded message
    // assert state fields
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-ui usage_tab_transitions_loading_progress_loaded -v`  
Expected: FAIL due to missing tab/state wiring.

**Step 3: Write minimal implementation**

- Add `DetailTab::Usage` next to `Volume`.
- Add `UsageUiState` for loading/progress/selected category/result payload.
- Add update handlers to trigger global scan and process progress/result messages.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-ui usage_tab_transitions_loading_progress_loaded -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-ui/src/ui/volumes/state.rs storage-ui/src/ui/volumes/message.rs storage-ui/src/ui/volumes/update.rs storage-ui/src/ui/volumes/mod.rs
git commit -m "feat(storage-ui): add Usage tab state and update flow"
```

---

### Task 6: Implement Usage tab view (bar chart + category tabs + grid + progress bar)

**Files:**
- Modify: `storage-ui/src/ui/volumes/view.rs` (or split subview module)
- Modify/Create: `storage-ui/src/ui/volumes/usage_pie.rs` (generalize segment type naming/shared input)
- Create/Modify: `storage-ui/src/ui/volumes/usage_bar.rs` (if separate bar renderer)
- Test: UI rendering tests where feasible

**Step 1: Write the failing test**

```rust
#[test]
fn usage_tab_renders_chart_tabs_and_file_grid_without_legend() {
    // render Usage tab with loaded state
    // assert bar chart present, legend hidden, tabs shown, grid columns path+size
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-ui usage_tab_renders_chart_tabs_and_file_grid_without_legend -v`  
Expected: FAIL due to missing view.

**Step 3: Write minimal implementation**

- Generalize pie segment input type to shared segment DTO adapter.
- Add bar chart using same segment list.
- Keep legend optional and disabled.
- Add color-coded tab control and grid (path + size) sorted by selected type.
- Show loading progress bar while loading.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-ui usage_tab_renders_chart_tabs_and_file_grid_without_legend -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-ui/src/ui/volumes/view.rs storage-ui/src/ui/volumes/usage_pie.rs storage-ui/src/ui/volumes/usage_bar.rs
git commit -m "feat(storage-ui): render Usage tab chart, tabs, and file grid"
```

---

### Task 7: End-to-end verification and docs note

**Files:**
- Modify: `README.md` (optional short feature note)

**Step 1: Run targeted package tests**

Run:
- `cargo test -p storage-common -v`
- `cargo test -p storage-dbus -v`
- `cargo test -p cosmic-ext-storage-storage-sys -v`
- `cargo test -p storage-ui -v`

Expected: all pass.

**Step 2: Run app smoke test**

Run app and verify:
- `Usage` tab appears next to `Volume`
- loading progress bar appears
- bar chart + category tabs + file grid render after completion.

**Step 3: Add short docs note (optional)**

Document global scope and that filtering is planned later.

**Step 4: Re-run quick regression test**

Run: `cargo test -p storage-ui -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add README.md
git commit -m "docs: note Usage tab global scan behavior"
```

---

## Notes for the Implementer

- Do not add mirror DTO definitions in `storage-sys`.
- Keep legend disabled by default; do not add extra UX beyond requested controls.
- Preserve chart color mapping consistency between pie/bar via shared segment semantics.
- Keep global-scan behavior explicit and stable for now.
