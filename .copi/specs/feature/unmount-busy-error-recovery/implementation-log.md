# Implementation Log: Unmount Resource Busy Error Recovery

**Branch:** `feature/unmount-busy-error-recovery`  
**Started:** 2026-02-11  
**Status:** In Progress (Task 4 of 7 completed)

---

## Progress Summary

| Task | Status | Notes |
|---|---|---|
| Task 1: Add Structured Error Type | ✅ Complete | DiskError::ResourceBusy added, detection working |
| Task 2: Implement Process Discovery | ✅ Complete | procfs-based discovery functional, tests pass |
| Task 3: Implement Process Termination | ✅ Complete | nix syscall implementation with safety checks |
| Task 4: Create Unmount Busy Dialog UI | ✅ Complete | Dialog renders with process list and warning |
| Task 5: Wire Dialog into Unmount Flow | ⏳ Next | Integration with actual unmount operations |
| Task 6: Add Logging and Error Context | ⏳ Pending | Will add after integration |
| Task 7: Documentation and Testing | ⏳ Pending | Final polish |

---

## Implementation Notes

### Task 1: Structured Error Type (Commit 69e76d1)
- Added `DiskError::ResourceBusy` variant with device and mount_point fields
- Implemented `check_resource_busy_error()` to detect EBUSY patterns:
  - "target is busy", "device is busy", "resource busy"
  - Case-insensitive matching
- Updated `fs_unmount()` to check for busy errors before generic error handling
- Added test for error pattern detection
- All existing tests continue to pass

**Decision:** Used pattern matching on error message strings rather than error codes since UDisks2 returns MethodError with descriptive messages

### Task 2: Process Discovery via procfs (Commit acf5d4b)
- Added `procfs = "0.16"` dependency
- Created `disks-dbus/src/disks/process_finder.rs` module
- Implemented `find_processes_using_mount()` with:
  - Async wrapper using `tokio::task::spawn_blocking`
  - Iteration over all processes via `/proc/[pid]/`
  - FD checking via `/proc/[pid]/fd/*` symlinks
  - Command extraction from cmdline/stat
  - Username resolution from `/etc/passwd`
- Edge cases handled:
  - Vanishing processes during iteration (silently skip)
  - Permission denied on FD access (skip that process)
  - Empty process list returns empty vec (not error)
- Tests added: username resolution, nonexistent mount handling

**Performance note:** Iterating all processes is O(n) but acceptable since unmount failures are rare and the operation is user-initiated

### Task 3: Process Termination via nix syscalls (Commit 1acefe4)
- Added `nix = { version = "0.29", features = ["signal"] }` dependency
- Implemented `kill_processes()` using `nix::sys::signal::kill(SIGKILL)`
- Returns `Vec<KillResult>` with per-PID outcomes
- Safety checks:
  - Refuse PID <= 1 (init/kernel processes)
  - ESRCH (process not found) → treated as success
  - EPERM (permission denied) → clear error message for user
- Synchronous implementation (no async overhead for simple syscall)
- Tests cover safety checks, nonexistent PIDs, negative PIDs

**Design note:** Immediate SIGKILL rather than graceful SIGTERM→SIGKILL because user is explicitly choosing destructive option with warning

**Permission model:** Application can only kill processes owned by same user. EPERM will be shown to user with advice to close manually

### Task 4: Unmount Busy Dialog UI (Commit pending)
- Added dialog state and message types:
  - `ShowDialog::UnmountBusy(UnmountBusyDialog)`
  - `UnmountBusyDialog` contains device, mount_point, processes
  - `UnmountBusyMessage` enum: Cancel, Retry, KillAndRetry
- Created dialog view in `disks-ui/src/ui/dialogs/view/mount.rs`
  - Process list displayed in scrollable column
  - PID, command, username shown in rows
  - Warning icon + text when processes exist
  - Three buttons: Cancel (tertiary), Retry (secondary), Kill+Retry (primary, destructive)
- Added localization strings to `i18n/en/cosmic_ext_disks.ftl`
- Wired dialog to view dispatcher in `app/view.rs`
- Added message handling stubs in `app/update/mod.rs`
- Exported ProcessInfo and related types from disks_dbus lib

**UI design decisions:**
- Used `button::destructive()` for Kill+Retry to emphasize danger
- Fixed height scrollable (200px) for process list
- Warning always shown when processes present
- If no processes found (edge case), Hide Kill button and show different message

**Build status:** Clean compilation, all tests pass, ready for integration

---

## Technical Decisions

### Why procfs over lsof?
- No external command dependency (lsof may not be installed)
- No privilege escalation needed for reading /proc
- Direct Rust API with better error handling
- Faster and more reliable parsing

### Why nix over shell commands for kill?
- Direct syscall is more efficient than spawning process
- Better error handling (errno codes vs exit codes)
- No polkit complexity
- Synchronous (appropriate for simple syscall)

### Dialog button ordering
Following COSMIC conventions:
- Tertiary (Cancel) - safe exit
- Secondary (Retry) - non-destructive action
- Primary (Kill + Retry) - destructive action with warning styling

---

## Next Steps (Task 5)

Wire the dialog into the actual unmount flow:
1. Modify `disks-ui/src/ui/volumes/update/mount.rs::unmount()`
2. Catch `DiskError::ResourceBusy` error
3. Get mount point from volume (may need to query UDisks2 for actual path)
4. Call `find_processes_using_mount()` 
5. Show `ShowDialog::UnmountBusy` dialog
6. Handle dialog responses:
   - Cancel: close dialog
   - Retry: call unmount again
   - KillAndRetry: call kill_processes(), then retry unmount

**Open question:** How to get actual mount point path from volume? Check if VolumeNode has mount_points field or query via DBus.

---

## Testing Strategy

### Completed Tests
- ✅ Error pattern detection (string matching)
- ✅ Username resolution from /etc/passwd
- ✅ Process discovery on nonexistent mount
- ✅ Kill safety checks (PID <= 1 rejection)
- ✅ Kill nonexistent PID handling
- ✅ All existing tests continue to pass

### Manual Testing Needed (Post-Integration)
- [ ] Create busy mount with open file: `cd /mnt/test && sleep 1000`
- [ ] Attempt unmount via UI
- [ ] Verify dialog shows sleep process
- [ ] Test Cancel: mount remains, dialog closes
- [ ] Test Retry: fails again (expected), dialog remains or shows error
- [ ] Test Kill + Retry: sleep killed, unmount succeeds
- [ ] Test with multiple processes holding mount
- [ ] Test with process owned by different user (EPERM scenario)

---

## Known Issues / TODOs

- [ ] Username resolution reads /etc/passwd on every call (consider caching if performance issue)
- [ ] Mount point display currently shows object_path as fallback - need actual mount path
- [ ] No retry limit on Retry button (user can click repeatedly) - acceptable for now
- [ ] Swedish translation not added yet (only English strings)
- [ ] Process list width is fixed - might not fit long usernames/commands

---

## Logs and Commands

### Build Commands Used
```bash
cargo test --workspace --all-features
cargo clippy --workspace --all-features
cargo build --workspace
```

### Test Results
- All 35 tests passing
- No clippy errors (warnings about pre-existing collapsible_if in smart.rs)
- Clean compilation

---

## Commits

1. `69e76d1` - feat(unmount): add structured error type for resource busy
2. `acf5d4b` - feat(unmount): implement process discovery via procfs
3. `1acefe4` - feat(unmount): implement process termination via nix syscalls
4. (pending) - feat(unmount): create unmount busy dialog UI component
