# Plan: Unmount Resource Busy Error Recovery

**Branch:** `feature/unmount-busy-error-recovery`  
**Source:** User brief (2026-02-11)  
**Status:** Spec created

---

## Context

When attempting to unmount a volume, the operation can fail with a "resource busy" error if files are open or processes are actively using the mounted filesystem. Currently, the application simply propagates the error to the user without providing actionable information or recovery options.

This creates a frustrating user experience because:
- Users don't know **what** is holding the mount open
- There's no way to identify and address the blocking processes
- Users must manually investigate using terminal commands
- No safe recovery path is provided within the UI

GNOME Disks and similar utilities provide enhanced error handling for this scenario, making the application more user-friendly and self-service capable.

---

## Goals

1. **Detect "resource busy" errors** during unmount operations
2. **Identify blocking processes** using `lsof` or equivalent system calls
3. **Present process information** to the user in a clear dialog
4. **Offer recovery options:**
   - Cancel (abort unmount)
   - Retry (attempt unmount again)
   - Kill + Retry (terminate processes and retry, with warning)
5. **Maintain safety** by warning users about potential data loss/corruption when killing processes
6. **Integrate seamlessly** with existing error handling and UI patterns

---

## Non-Goals

- Handling other types of unmount errors beyond "resource busy" (those continue with existing error display)
- Automatic process killing without user confirmation
- Cross-platform support (Linux-only per repo conventions)
- Graceful process termination (SIGTERM then SIGKILL) — use immediate kill (SIGKILL) with clear warning
- Handling kernel-level locks or NFS mounts (out of scope for initial implementation)

---

## Proposed Approach

### A) Error Detection Layer (disks-dbus)

Enhance `disks-dbus/src/disks/ops.rs` unmount error handling:
1. Parse UDisks2 error response to detect "target is busy" / EBUSY
2. Return a structured error variant (e.g., `DiskError::ResourceBusy { device: String, mount_point: String }`)
3. Keep existing generic error handling for other failure modes

### B) Process Discovery (disks-dbus or utils)

Add system integration for finding blocking processes using the **`procfs` crate**:
1. Enumerate all processes via `/proc/[pid]/`
2. For each process, read `/proc/[pid]/fd/*` symlinks to find open file descriptors
3. Check if any fd points to a path under the target mount point
4. Collect process info: PID, command name (from `/proc/[pid]/cmdline`), UID (from `/proc/[pid]/status`)
5. Return structured data: `Vec<ProcessInfo>` with PID, command name, user

**Advantages over lsof:**
- No external command dependency (lsof may not be installed)
- No need for privilege escalation to query process info
- Faster and more reliable parsing
- Direct Rust API with proper error handling

### C) Process Termination (disks-dbus)

Add utility to kill processes by PID using direct syscalls:
1. Use `nix::sys::signal::kill()` with `Signal::SIGKILL`
2. Return success/failure for each PID
3. Handle permission errors gracefully (EPERM if we don't own the process)
4. Safety check: validate PID > 1 (never kill init/kernel threads)

**Advantages:**
- No shell command spawning
- Direct syscall is more efficient
- Better error handling (errno codes)
- No polkit complexity for the kill operation itself

### D) UI Dialog (disks-ui)

Create a new dialog in `disks-ui/src/ui/dialogs/`:
1. **Dialog type:** Modal, blocking (similar to password prompt)
2. **Content sections:**
   - **Title:** "Unable to Unmount — Device is Busy"
   - **Body:** "The following processes are accessing {mount_point}:"
   - **Process list:** Scrollable list showing PID, command, user (table or list widget)
   - **Warning (if Kill selected):** Destructive warning text with icon
3. **Action buttons:**
   - **Cancel** (default, neutral)
   - **Retry** (secondary)
   - **Kill + Retry** (destructive style, warning color)

### E) Integration

Wire the error flow:
1. `mount.rs` unmount operation catches `ResourceBusy` error
2. Async task: invoke process discovery via dbus
3. Show dialog via `Message::ShowUnmountBusyDialog { processes, mount_point }`
4. Dialog response:
   - Cancel → no-op
   - Retry → retry unmount
   - Kill + Retry → kill processes, then retry unmount
5. If kill or retry fails, show standard error dialog

---

## User/System Flows

### Flow 1: Normal Unmount (Happy Path)
1. User clicks "Unmount" on mounted volume
2. UDisks2 unmount succeeds
3. UI updates showing volume as unmounted

### Flow 2: Resource Busy Unmount (New Behavior)
1. User clicks "Unmount" on mounted volume
2. UDisks2 returns EBUSY error
3. System detects "resource busy" and queries lsof for processes
4. Dialog appears: "Unable to Unmount — Device is Busy"
5. User sees process list (example):
   ```
   PID    Command         User    
   1234   nautilus        alice   
   5678   vim             alice   
   ```
6. **Option A: User clicks "Cancel"** → Dialog closes, mount remains
7. **Option B: User clicks "Retry"** → Unmount attempted again (likely fails unless user manually closed files)
8. **Option C: User clicks "Kill + Retry"**
   - Warning highlight appears on button/dialog
   - User confirms (or button is already in destructive style)
   - System kills PIDs 1234, 5678
   - System retries unmount
   - If successful, UI updates; if failed, show error dialog

### Flow 3: Process Discovery Fails or Returns Empty
1. Resource busy error detected
2. procfs query fails (unlikely on Linux) or returns no processes (rare edge case)
3. Show simplified dialog: "Device is busy but no processes found. Retry or cancel?"
4. Buttons: Cancel, Retry (no Kill option)

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| **Kill causes data corruption** | High | Clear warning text, destructive button styling, require explicit user action |
| **procfs read permission denied** | Low | Unlikely on Linux; fail gracefully with simplified dialog |
| **Process re-opens file after kill** | Low | Document that Retry may fail anyway; this is expected behavior |
| **Dialog adds complexity** | Low | Keep design simple, reuse existing dialog patterns from COSMIC toolkit |
| **Race condition (process exits before kill)** | Low | Ignore kill errors for non-existent PIDs |

---

## Acceptance Criteria

- [ ] Unmount operation detects "resource busy" errors from UDisks2
- [ ] System identifies processes holding mount point open via procfs
- [ ] Dialog displays process information (PID, command, user) in readable format
- [ ] Dialog offers three clear options: Cancel, Retry, Kill + Retry
- [ ] Kill + Retry button has destructive/warning styling
- [ ] Dialog includes warning text about data loss when Kill is an option
- [ ] Killing processes executes correctly (requires appropriate permissions)
- [ ] After kill, unmount is automatically retried
- [ ] If retry fails, standard error dialog is shown
- [ ] Manual testing: Create busy mount (e.g., `cd /mount && sleep 1000`), attempt unmount, verify dialog and all three actions
- [ ] Code follows repo conventions (Rust 2024, clippy clean, formatted)

---

## Open Questions

1. **Privilege escalation:** Can the app kill processes owned by other users?
   - procfs reading doesn't require privileges
   - kill syscall succeeds only for processes owned by the same user (or root)
   - If we can't kill a process (EPERM), inform user they may need to close it manually or run as root
   - **Decision:** Document limitation in UI ("Some processes could not be killed. Try closing them manually.")

2. **Process list display:** Should we limit to top N processes or show all?
   - **Recommendation:** Show all, with scrollable list if >5 processes

3. **Graceful shutdown:** Should we try SIGTERM before SIGKILL?
   - **Decision:** No, use SIGKILL only (simpler, user explicitly chose destructive option)
   - Can be revisited in future if users request it

4. **Retry limit:** Should we limit retry attempts to avoid infinite loops?
   - **Decision:** No automatic limit; user controls retry via button clicks

---

## Related Work

- **Audit references:** None (user brief)
- **Related specs:** 
  - `.copi/specs/feature/luks-logical-volumes/` (unmount of children)
  - `.copi/specs/fix/luks-delete-preflight/` (unmount before delete)
- **Upstream references:** GNOME Disks "Failed to unmount" dialogs, Dolphin busy device handling

---

## Dependencies

- **Crates:** 
  - `procfs` (v0.16+) for direct /proc filesystem parsing
  - `nix` (v0.29+) for kill syscall (`nix::sys::signal::kill`)
- **System tools:** None (pure Rust implementation)
- **Polkit:** May be needed only if killing processes owned by other users (application likely runs as user, can only kill own processes)

---

## Success Metrics

- Unmount busy errors are resolved in <30 seconds by users (vs. manual investigation)
- Zero crashes or panics related to process killing edge cases
- Clear user feedback in issue reports or reviews about improved unmount UX
