# Usage Action Bar Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a Usage action/settings bar with admin-gated all-files listing, Shift/Ctrl selection semantics, delete/clear actions, configurable top-file count, and explicit refresh-driven scans.

**Architecture:** Extend the existing Usage tab state/message/update pipeline (Approach A) without introducing a new UI subsystem. Service API is expanded to accept scan controls and provide delete operations; UI drives all behavior through existing app update flow and `UsageTabState`. Selection is managed by stable full-path keys with anchor-based range operations.

**Tech Stack:** Rust 2024 workspace, `storage-ui` (COSMIC/iced UI), `storage-service` + `zbus` DBus interfaces, `storage-sys` scanner pipeline, `storage-common` shared DTOs, serde JSON.

---

### Task 1: Extend shared API models for scan controls and delete results

**Files:**
- Modify: `storage-common/src/usage_scan.rs`
- Modify: `storage-common/src/lib.rs`
- Test: `storage-common/src/usage_scan.rs` (unit tests module)

**Step 1: Write the failing test**

```rust
#[test]
fn usage_scan_request_and_delete_result_roundtrip() {
    let request = UsageScanRequest {
        scan_id: "scan-1".into(),
        top_files_per_category: 20,
        show_all_files: false,
    };
    let json = serde_json::to_string(&request).expect("serialize request");
    let parsed: UsageScanRequest = serde_json::from_str(&json).expect("parse request");
    assert_eq!(parsed.scan_id, "scan-1");

    let result = UsageDeleteResult {
        deleted: vec!["/tmp/a".into()],
        failed: vec![UsageDeleteFailure {
            path: "/tmp/b".into(),
            reason: "permission denied".into(),
        }],
    };
    let json = serde_json::to_string(&result).expect("serialize result");
    let parsed: UsageDeleteResult = serde_json::from_str(&json).expect("parse result");
    assert_eq!(parsed.deleted.len(), 1);
    assert_eq!(parsed.failed.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p storage-common usage_scan_request_and_delete_result_roundtrip -v`  
Expected: FAIL (types missing).

**Step 3: Write minimal implementation**

Add:
- `UsageScanRequest { scan_id, top_files_per_category, show_all_files }`
- `UsageDeleteFailure { path, reason }`
- `UsageDeleteResult { deleted, failed }`

Derive `Serialize`, `Deserialize`, `Debug`, `Clone`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p storage-common usage_scan_request_and_delete_result_roundtrip -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-common/src/usage_scan.rs storage-common/src/lib.rs
git commit -m "feat(storage-common): add usage scan request and delete result DTOs"
```

---

### Task 2: Add scanner-side ownership/access filtering support

**Files:**
- Modify: `storage-sys/src/usage/types.rs`
- Modify: `storage-sys/src/usage/scanner.rs`
- Modify: `storage-sys/src/usage/mod.rs`
- Test: `storage-sys/src/usage/scanner.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn scan_respects_user_only_filter() {
    let config = ScanConfig {
        threads: Some(1),
        top_files_per_category: 5,
        show_all_files: false,
    };
    // Build fixture paths with synthetic metadata helper where available.
    // Assert scanner includes only user-accessible entries under default mode.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage-storage-sys scan_respects_user_only_filter -v`  
Expected: FAIL (no show-all/user-only support).

**Step 3: Write minimal implementation**

- Add `show_all_files: bool` to `ScanConfig` (default `false`).
- In file traversal, when `show_all_files == false`, include only files owned/accessibly readable by caller UID.
- Keep behavior DRY and isolated in a helper (avoid duplicate checks in traversal loop).

**Step 4: Run test to verify it passes**

Run: `cargo test -p cosmic-ext-storage-storage-sys scan_respects_user_only_filter -v`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-sys/src/usage/types.rs storage-sys/src/usage/scanner.rs storage-sys/src/usage/mod.rs
git commit -m "feat(storage-sys): add user-only filtering for usage scans"
```

---

### Task 3: Extend Filesystems DBus scan API with controls and add delete endpoint

**Files:**
- Modify: `storage-service/src/filesystems.rs`
- Modify: `storage-ui/src/client/filesystems.rs`
- Test: `storage-ui/src/client/filesystems.rs` (client parse tests if present)

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn client_usage_scan_accepts_show_all_flag() {
    // Stub JSON/proxy response if test harness supports.
    // Assert method signature and parsing for updated API call.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage client_usage_scan_accepts_show_all_flag -v`  
Expected: FAIL (signature mismatch / missing delete method).

**Step 3: Write minimal implementation**

- Service method: `get_usage_scan(scan_id, top_files_per_category, show_all_files)`.
- Wire `show_all_files` into scanner config.
- Add service method `delete_usage_files(paths_json: String) -> String` returning `UsageDeleteResult` JSON.
- Client proxy mirrors both signatures and parses `UsageDeleteResult`.

**Step 4: Run test to verify it passes**

Run: `cargo check -p storage-service -p cosmic-ext-storage`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-service/src/filesystems.rs storage-ui/src/client/filesystems.rs
git commit -m "feat(filesystems): add usage scan controls and delete endpoint"
```

---

### Task 4: Add Usage action bar state + messages in UI model

**Files:**
- Modify: `storage-ui/src/ui/volumes/state.rs`
- Modify: `storage-ui/src/ui/app/message.rs`
- Modify: `storage-ui/src/ui/app/update/mod.rs`
- Test: `storage-ui/src/ui/app/update/mod.rs` (or closest update tests)

**Step 1: Write the failing test**

```rust
#[test]
fn usage_control_state_transitions() {
    // show_all_files toggle, top_files_per_category edit,
    // refresh request, clear selection.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage usage_control_state_transitions -v`  
Expected: FAIL.

**Step 3: Write minimal implementation**

Add usage-state fields:
- `show_all_files`, `show_all_files_authorized_for_session`
- `top_files_per_category`
- `selected_paths`
- `selection_anchor_index`
- `deleting`

Add messages for:
- toggles/inputs/refresh
- selection operations (single/ctrl/shift)
- delete start/completion

**Step 4: Run test to verify it passes**

Run: `cargo check -p cosmic-ext-storage`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-ui/src/ui/volumes/state.rs storage-ui/src/ui/app/message.rs storage-ui/src/ui/app/update/mod.rs
git commit -m "feat(storage-ui): add usage action bar and selection state/messages"
```

---

### Task 5: Implement action bar UI + selectable file list interactions

**Files:**
- Modify: `storage-ui/src/ui/app/view.rs`
- Test: `storage-ui/src/ui/app/view.rs` (snapshot/manual + compile)

**Step 1: Write the failing test**

```rust
#[test]
fn usage_view_renders_action_bar_controls() {
    // Ensure Show All Files, Delete, Clear Selection,
    // Number of files, Refresh are all present.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage usage_view_renders_action_bar_controls -v`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- Add action/settings bar above category tabs.
- Bind controls to new messages.
- Render selected-row styling and click handlers implementing single/Shift/Ctrl behavior.
- Keep filename column + tooltip path behavior.
- Keep existing usage totals/bar behavior.

**Step 4: Run test to verify it passes**

Run: `cargo check -p cosmic-ext-storage`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-ui/src/ui/app/view.rs
git commit -m "feat(storage-ui): render usage action bar and selectable rows"
```

---

### Task 6: Wire auth flow for Show All Files and mixed Delete behavior

**Files:**
- Modify: `storage-ui/src/ui/app/update/mod.rs`
- Modify: `storage-service/src/filesystems.rs`
- Test: service auth branching tests + UI update transitions

**Step 1: Write the failing test**

```rust
#[test]
fn show_all_toggle_reverts_on_auth_cancel() {
    // toggle on -> auth denied -> state remains off
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p cosmic-ext-storage show_all_toggle_reverts_on_auth_cancel -v`  
Expected: FAIL.

**Step 3: Write minimal implementation**

- On `Show All Files` toggle-on, perform privileged probe/authorization call.
- Cache successful auth for session.
- On delete, if any selected path privileged, request admin then execute mixed delete.
- Preserve selection on delete failure/cancel.

**Step 4: Run test to verify it passes**

Run: `cargo check -p storage-service -p cosmic-ext-storage`  
Expected: PASS.

**Step 5: Commit**

```bash
git add storage-ui/src/ui/app/update/mod.rs storage-service/src/filesystems.rs
git commit -m "feat(auth): gate show-all and mixed delete with admin authorization"
```

---

### Task 7: Final verification and regression pass

**Files:**
- Modify (if needed): `README.md` (short note for usage controls)

**Step 1: Run targeted checks**

Run:
- `cargo check -p storage-common -p cosmic-ext-storage-storage-sys -p storage-service -p cosmic-ext-storage`
- `cargo test -p cosmic-ext-storage-storage-sys -v`

Expected: PASS.

**Step 2: Manual smoke checks**

Verify:
- action bar controls all render and function,
- refresh applies `show_all_files` + `top_files_per_category`,
- selection model supports single/Shift/Ctrl,
- delete + clear selection behavior,
- filename + tooltip path remains correct.

**Step 3: Commit any doc touchups**

```bash
git add README.md
git commit -m "docs: describe usage action bar controls"
```

---

## Notes for Implementer
- Keep existing progress/totals/bar rendering intact except where action bar and row-selection behavior require augmentation.
- Avoid adding new pages/modals; stay in current Usage tab.
- Follow YAGNI: no bulk background jobs, no new filtering dimensions, no speculative cache layer.
- Keep operations explicit: control edits are inert until `Refresh`.
