# Tasks: Unmount Resource Busy Error Recovery

**Branch:** `feature/unmount-busy-error-recovery`  
**Source:** User brief (2026-02-11)

---

## Overview

This document breaks down the "resource busy" unmount error recovery feature into commit/small-PR sized tasks. Tasks should be completed in the order listed to maintain a working build at each commit.

Dependencies:
- Task 1 → Task 2 (error type needed for detection)
- Task 2 → Task 3 (detection needed before process discovery)
- Task 3 → Task 4 (process data needed for dialog)
- Task 4 → Task 5 (dialog needed before integration)

---

## Task 1: Add Structured Error Type for Resource Busy

**Scope:** Extend error types in `storage-dbus` to distinguish "resource busy" from generic errors.

**Files/Areas:**
- `storage-dbus/src/disks/ops.rs` (error handling in `fs_unmount`)
- Consider: add `DiskError` enum or extend existing error handling

**Steps:**
1. Review current error handling in `fs_unmount` (line ~242-270)
2. Identify UDisks2 error message patterns for EBUSY:
   - Check zbus error messages for "target is busy", "device is busy", etc.
   - May need to inspect `zbus::Error::MethodError` message content
3. Create structured error variant:
   ```rust
   pub enum UnmountError {
       ResourceBusy { device: String, mount_point: Option<String> },
       Other(anyhow::Error),
   }
   ```
   Or extend existing `anyhow::Error` with context
4. Update `fs_unmount` to return structured error when EBUSY detected
5. Ensure `partition_unmount` propagates the new error type
6. Add unit test: mock UDisks2 error, verify correct error variant returned

**Test Plan:**
- Unit test with mocked "device busy" error message
- Verify error propagates correctly through `partition_unmount`
- Build and clippy pass

**Done When:**
- [ ] Error type distinguishes "resource busy" from other failures
- [ ] `fs_unmount` returns structured error on EBUSY
- [ ] Unit test covers detection logic
- [ ] Code formatted and clippy clean

---

## Task 2: Implement Process Discovery via procfs

**Scope:** Add system utility to find processes holding a mount point open using the procfs crate.

**Files/Areas:**
- New file: `storage-dbus/src/disks/process_finder.rs` or similar
- `storage-dbus/src/disks/mod.rs` (module declaration)
- `storage-dbus/Cargo.toml` (add procfs dependency)

**Steps:**
1. Add `procfs = "0.16"` to `storage-dbus/Cargo.toml`
2. Create module with public function:
   ```rust
   pub struct ProcessInfo {
       pub pid: i32,
       pub command: String,
       pub uid: u32,
       pub username: String,
   }
   
   pub async fn find_processes_using_mount(mount_point: &str) -> Result<Vec<ProcessInfo>>
   ```
3. Implement using procfs crate:
   ```rust
   // Iterate all processes
   for process in procfs::process::all_processes()? {
       let process = process?;
       // Check each fd symlink
       if let Ok(fds) = process.fd() {
           for fd in fds {
               if let Ok(target) = fd.target() {
                   // Check if target path is under mount_point
                   if target.starts_with(mount_point) {
                       // Collect process info
                   }
               }
           }
       }
   }
   ```
4. Extract process details:
   - PID from `process.pid()`
   - Command from `process.cmdline()?.join(" ")` or `process.stat()?.comm`
   - UID from `process.status()?.ruid`
   - Username: resolve UID to username via `/etc/passwd` or `users` crate
5. Handle edge cases:
   - Process vanishes during iteration (ENOENT) → skip silently
   - Permission denied on fd access → skip that process
   - No processes found → return empty vec
6. Make async-compatible (wrap blocking procfs calls in `tokio::task::spawn_blocking`)
7. Add logging for each outcome (tracing::debug/warn)
8. Write unit test with mocked procfs data (or integration test)

**Test Plan:**
- Integration test: open file in temp location, verify process is found
- Test with multiple processes holding same mount
- Test permission denied handling (skip process gracefully)
- Verify no panic on process disappearing during iteration

**Done When:**
- [ ] `find_processes_using_mount` returns structured process list
- [ ] procfs iteration handles all edge cases (vanishing processes, permission denied)
- [ ] Username resolution works correctly
- [ ] Async wrapper (spawn_blocking) implemented
- [ ] Edge cases handled gracefully (no panic)
- [ ] Tests pass, code formatted

---

## Task 3: Implement Process Termination Utility

**Scope:** Add utility function to kill processes by PID using syscalls.

**Files/Areas:**
- Same module as Task 2: `storage-dbus/src/disks/process_finder.rs` or separate module
- `storage-dbus/Cargo.toml` (add nix dependency)

**Steps:**
1. Add `nix = { version = "0.29", features = ["signal"] }` to `storage-dbus/Cargo.toml`
2. Add public function:
   ```rust
   pub struct KillResult {
       pub pid: i32,
       pub success: bool,
       pub error: Option<String>,
   }
   
   pub fn kill_processes(pids: &[i32]) -> Vec<KillResult>
   ```
   Returns per-PID success/failure
3. Implement using `nix::sys::signal::kill`:
   ```rust
   use nix::sys::signal::{kill, Signal};
   use nix::unistd::Pid;
   
   for &pid in pids {
       // Safety check
       if pid <= 1 {
           results.push(KillResult {
               pid,
               success: false,
               error: Some("Refusing to kill system process".into()),
           });
           continue;
       }
       
       match kill(Pid::from_raw(pid), Signal::SIGKILL) {
           Ok(()) => results.push(KillResult { pid, success: true, error: None }),
           Err(e) => results.push(KillResult { 
               pid, 
               success: false, 
               error: Some(e.to_string()),
           }),
       }
   }
   ```
4. Handle edge cases:
   - PID doesn't exist (ESRCH) → treat as success (already gone)
   - Permission denied (EPERM) → return error with clear message
   - Invalid PID → return error for that PID
5. Add logging for each kill attempt (tracing::debug for success, warn for failures)
6. No async needed (kill is synchronous syscall)

**Test Plan:**
- Unit test: verify safety checks (PID <= 1 rejected)
- Integration test: spawn test process, kill it via syscall
- Test invalid PID handling (negative, 0, very large)
- Test non-existent PID (ESRCH should be handled gracefully)
- Test permission scenarios (if feasible)

**Done When:**
- [ ] `kill_processes` terminates PIDs with SIGKILL via nix syscall
- [ ] Per-PID results returned with success/error status
- [ ] Safety checks prevent killing system processes (PID <= 1)
- [ ] ESRCH (process not found) treated as success
- [ ] EPERM (permission denied) reported clearly
- [ ] No async overhead (synchronous syscall)
- [ ] Tests pass, code formatted

---

## Task 4: Create Unmount Busy Dialog UI Component

**Scope:** Build the modal dialog for displaying busy error and recovery options.

**Files/Areas:**
- New file: `storage-ui/src/ui/dialogs/unmount_busy.rs`
- Update: `storage-ui/src/ui/dialogs/mod.rs` (module declaration)

**Steps:**
1. Define dialog state struct:
   ```rust
   pub struct UnmountBusyDialog {
       pub mount_point: String,
       pub processes: Vec<ProcessInfo>,
   }
   ```
2. Add message variants in `storage-ui/src/ui/dialogs/message.rs`:
   ```rust
   enum DialogMessage {
       UnmountBusy(UnmountBusyAction),
   }
   
   enum UnmountBusyAction {
       Cancel,
       Retry,
       KillAndRetry,
   }
   ```
3. Implement dialog view function:
   - Title: "Unable to Unmount — Device is Busy"
   - Body text: "The following processes are accessing {mount_point}:"
   - Process list (scrollable if >5):
     - Use `widget::column` or `widget::scrollable`
     - Show PID, command, user in rows (or simple list)
   - Warning section (conditional on Kill option visibility):
     - Icon + text: "Killing processes may cause data loss or corruption."
   - Button row:
     - Cancel (default)
     - Retry (secondary)
     - Kill + Retry (destructive style via `cosmic::theme::Button::Destructive`)
4. Apply COSMIC widget styling (match existing dialogs in dialogs/)
5. Handle empty process list edge case:
   - Show "No processes found" message
   - Only show Cancel and Retry buttons

**Test Plan:**
- Visual test: manually trigger dialog with mock data
- Verify layout with 1, 3, 10 processes
- Verify button actions send correct messages
- Test with empty process list

**Done When:**
- [ ] Dialog renders with process list
- [ ] All three buttons present and styled correctly
- [ ] Kill + Retry has destructive/warning appearance
- [ ] Warning text displayed
- [ ] Empty process list handled gracefully
- [ ] Code follows UI patterns in `dialogs/` directory

---

## Task 5: Wire Dialog into Unmount Error Flow

**Scope:** Integrate process discovery and dialog into existing unmount operations.

**Files/Areas:**
- `storage-ui/src/ui/volumes/update/mount.rs` (unmount function)
- `storage-ui/src/ui/volumes/message.rs` (add message variants)
- `storage-ui/src/app.rs` or dialog management (show dialog)

**Steps:**
1. Update `unmount` function in `mount.rs`:
   - Modify `perform_volume_operation` to handle `UnmountError::ResourceBusy`
   - On busy error, invoke process discovery:
     ```rust
     let mount_point = volume.get_mount_point()?;
     let processes = process_finder::find_processes_using_mount(&mount_point).await?;
     ```
   - Return message to show dialog:
     ```rust
     Message::ShowUnmountBusyDialog { mount_point, processes, volume_path }
     ```
2. Add message handler in app or dialog controller:
   - On `ShowUnmountBusyDialog`: store dialog state, set dialog visible
3. Handle dialog actions:
   - **Cancel:** Close dialog, no-op
   - **Retry:** Call unmount again (same as original operation)
   - **KillAndRetry:**
     1. Extract PIDs from processes
     2. Call `kill_processes(pids)` (synchronous)
     3. Check results: if any EPERM errors, show warning "Some processes could not be killed (permission denied)"
     4. Call unmount again
     5. If unmount still fails, show generic error dialog
4. Update error propagation for non-busy errors (continue existing behavior)
5. Add retry limit or guard against infinite loops (optional: track retry count)

**Test Plan:**
- Manual test: create busy mount, attempt unmount
  - Setup: `mkdir /tmp/testmount && sudo mount -t tmpfs tmpfs /tmp/testmount && (cd /tmp/testmount && sleep 1000) &`
  - Click unmount in UI
  - Verify dialog appears with `sleep` process
  - Test Cancel: mount remains
  - Test Retry: fails again (expected)
  - Test Kill + Retry: sleep killed, unmount succeeds
- Test non-busy errors still show generic error dialog
- Test empty process list case

**Done When:**
- [ ] Unmount busy error triggers dialog
- [ ] Process list displayed in dialog
- [ ] Cancel, Retry, Kill + Retry all work correctly
- [ ] Kill + Retry successfully unmounts after killing processes
- [ ] Non-busy errors continue with existing error handling
- [ ] Manual test scenarios pass
- [ ] No regressions in normal unmount flow

---

## Task 6: Add Logging and Error Context

**Scope:** Ensure all steps are logged for debugging and user support.

**Files/Areas:**
- All modified files from Tasks 1-5

**Steps:**
1. Add `tracing::debug` logs:
   - When resource busy detected
   - When procfs query starts/completes
   - When processes are killed (per-PID results)
   - When retry unmount starts
2. Add `tracing::warn` for edge cases:
   - procfs read failed
   - Permission denied (EPERM) when killing
   - Kill failed for specific PID
3. Add `tracing::error` for unexpected failures
4. Include context in all logs: mount_point, PID list, error messages
5. Review existing error messages in UI for clarity
6. Ensure errors are preserved in log file per existing logging setup

**Test Plan:**
- Run manual test from Task 5
- Verify logs appear in console/log file
- Check log messages are clear and actionable

**Done When:**
- [ ] All major operations logged at appropriate levels
- [ ] Error context useful for debugging
- [ ] Logs reviewed for clarity

---

## Task 7: Documentation and Testing

**Scope:** Document the feature and add integration tests.

**Files/Areas:**
- `README.md` (optional: mention unmount recovery feature)
- `storage-ui/README.md` (if exists)
- `storage-dbus/src/disks/process_finder.rs` (doc comments)
- New integration test (optional)

**Steps:**
1. Add rustdoc comments to all public functions in process_finder module
2. Document error types and when they're returned
3. Update README if feature is user-facing enough (low priority)
4. Add integration test (if feasible):
   - Spin up test environment with busy mount
   - Trigger unmount busy scenario
   - Verify process discovery works
   - (Optional) Test full kill + retry flow
5. Review all clippy warnings and address
6. Run `cargo test --workspace --all-features`
7. Run `cargo clippy --workspace --all-features`
8. Run `cargo fmt --all --check`

**Test Plan:**
- All tests pass
- Clippy clean
- Formatted correctly
- CI passes (if running GitHub Actions)

**Done When:**
- [ ] Public APIs documented
- [ ] Integration test added (or documented why not feasible)
- [ ] Clippy and fmt pass
- [ ] All tests pass

---

## Optional/Future Enhancements (Not in Scope)

- Graceful termination (SIGTERM → wait → SIGKILL) instead of immediate SIGKILL
- Support for NFS/CIFS mounts with network-specific busy detection
- Automatic retry with exponential backoff
- Process tree display (show child processes under parent)
- User-friendly process names (e.g., "File Manager" instead of "nautilus")
- Detection of kernel-level locks (via /proc/locks)
- Show additional details per process (cwd, open file paths, user-friendly descriptions)

---

## Testing Strategy Summary

**Unit Tests:**
- Error type detection (mock UDisks2 errors)
- procfs parsing logic (if unit-testable)
- Kill safety checks (reject PID <= 1)

**Integration Tests:**
- Process discovery on real temp mount with open file
- Verify process found correctly via procfs
- Full unmount busy flow (setup, trigger, verify)

**Manual Tests (Critical):**
- Create busy mount with sleep process
- Verify dialog appears and shows correct processes
- Test all three actions (Cancel, Retry, Kill + Retry)
- Verify unmount succeeds after Kill + Retry
- Test edge case: empty process list
- Test edge case: process vanishes during iteration

**Regression Tests:**
- Normal unmount still works
- Non-busy errors show generic error dialog
- Other volume operations unaffected

---

## Estimated Effort

| Task | Estimated Time | Complexity |
|---|---|---|
| Task 1 | 1-2 hours | Low |
| Task 2 | 2-3 hours | Medium |
| Task 3 | 1-2 hours | Low |
| Task 4 | 2-4 hours | Medium (UI) |
| Task 5 | 2-3 hours | Medium |
| Task 6 | 1 hour | Low |
| Task 7 | 1-2 hours | Low |
| **Total** | **10-17 hours** | **Medium** |

Note: Assumes familiarity with COSMIC toolkit and codebase. First-time contributors may need additional time for setup and learning.

---

## Completion Checklist

- [ ] All tasks completed in order
- [ ] Manual testing scenarios pass
- [ ] CI checks pass (tests, clippy, fmt)
- [ ] No regressions in existing unmount behavior
- [ ] Logs are clear and useful
- [ ] Code reviewed (self or peer)
- [ ] Ready for PR against `main`
