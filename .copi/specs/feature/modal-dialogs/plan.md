# Modal Dialog Windows — Plan

**Branch:** N/A (deferred)  
**Source:** User request (addresses README V1 goal #1 "Dialogs are still ugly")  
**Status:** ⛔ **DEFERRED** — Waiting for upstream libcosmic support

---

## ⚠️ Research Update (2026-02-12)

**CRITICAL FINDING:** Research (see [research-findings.md](research-findings.md)) has revealed that **libcosmic does not currently support parent-child window relationships or true modal window semantics**. All windows created with `window::open()` are independent top-level windows without parenting capabilities.

**Impact:** The original spec goal ("Convert all dialogs to separate OS-level modal windows") is **not feasible** with the current COSMIC framework.

**Spec Status:** DEFERRED — Waiting for upstream libcosmic support

**Decision:** Option A (Wait for Upstream) chosen. This spec is blocked until libcosmic implements parent-child window relationships and modal window semantics.

**Alternative considered:** Enhanced overlay dialogs (Option B) was recommended but user decided to wait for proper modal window support.

**Next Step:** Monitor libcosmic development for modal window feature. Revisit when feature becomes available (expected: Q3 2026 or later).

---

## Context

> **Note:** The below context describes the *ideal end-state* with true modal windows. Implementation approach will differ based on product decision above.

The application currently implements dialogs using `cosmic::widget::dialog`, which renders as an overlay on top of the main window. This approach has several limitations:
- Dialogs cannot be moved or resized independently
- They block the main window but aren't true OS-level modal windows
- Users cannot interact with other applications while a dialog is active
- The UX feels constrained compared to native desktop applications
- All 17 dialog types currently suffer from this overlay presentation

This represents a gap between the application's current state and parity with established disk utilities like GNOME Disks, where critical operations (partition creation, formatting, etc.) open in proper modal dialogs.

**Referenced Documentation:**
- Current dialog implementation: [disks-ui/src/ui/dialogs/view/common.rs](../../../disks-ui/src/ui/dialogs/view/common.rs)
- Dialog state enum: [disks-ui/src/ui/dialogs/state.rs](../../../disks-ui/src/ui/dialogs/state.rs) (17 dialog types)
- Dialog rendering: [disks-ui/src/ui/app/view.rs](../../../disks-ui/src/ui/app/view.rs#L28-L119)
- README V1 goal #1: [README.md](../../../README.md#L26)
- Related audit finding: [GAP-004 Dialog State Management](.copi/audits/2026-02-06T17-26-25Z.md#L276) (internal refactoring, out of scope here)

---

## Goals

1. **Convert all dialogs to separate OS-level windows** instead of overlay widgets
2. **Make dialogs modal** (blocking parent window interaction while open)
3. **Allow dialog windows** to be:
   - Moved independently on screen
   - Positioned relative to parent window (centered by default)
   - Closed via window controls (X button) in addition to Cancel
4. **Preserve all existing dialog functionality**:
   - Form validation
   - Async operation states (running indicators)
   - Error display
   - Custom layouts for complex dialogs (partition creation, SMART data, etc.)
5. **Improve dialog UX** as a side effect (proper window management, OS integration)

---

## Non-Goals

- **Dialog system internal refactoring**: While GAP-004 suggests state management improvements, this spec focuses only on window presentation, not internal architecture
- **Dialog visual redesign**: Keep existing layouts and styling, only change the container from overlay to window
- **Animation/transitions**: Focus on functionality, not visual polish
- **Other UI improvements**: Sidebar, context drawer, main view layout changes are out of scope
- **Platform-specific optimizations**: Linux/COSMIC only (per repo rules)

---

## Proposed Approach

### A) COSMIC Window Management Research

**Critical first step:** Determine how COSMIC applications spawn modal child windows.

**Potential approaches:**
1. **Multi-window Application Pattern** (preferred if supported):
   - Use `cosmic::Application` with multiple window IDs
   - Each dialog spawns as a separate window with parent window ID set
   - COSMIC runtime handles modality and window parenting
   - State management remains centralized in `AppModel`

2. **iced native window spawning**:
   - Use underlying iced runtime to spawn additional windows
   - May require deeper integration with COSMIC compositor

3. **Separate Dialog Process** (fallback):
   - Launch dialog as subprocess with IPC (message passing)
   - More complex but guarantees OS-level separation
   - Higher implementation cost

**Research sources:**
- libcosmic documentation and source code
- Example COSMIC applications (cosmic-settings, cosmic-files, etc.)
- Community channels (Matrix, Discord)

---

### B) Implementation Path

**Phase 1: Research & Proof of Concept (Tasks 1-2)**
1. Investigate COSMIC window management APIs
2. Implement simplest dialog (Info dialog) as separate window
3. Validate approach: modality, window lifecycle, state management

**Phase 2: Infrastructure (Task 3)**
1. Create dialog window lifecycle management
2. Generalize PoC to support all dialog types
3. Define message routing for multi-window app

**Phase 3: Migration (Tasks 4-10)**
Migrate each dialog category systematically:
- Confirmation dialogs (delete, generic confirms)
- Partition dialogs (create, edit, resize)
- Format dialogs (partition, disk, label)
- Encryption dialogs (unlock, change passphrase, options)
- Mount options dialogs
- Image dialogs (create, attach, operations)
- SMART and unmount-busy dialogs

**Phase 4: Cleanup (Task 11)**
- Remove legacy overlay rendering code
- Update tests
- Full validation across all 17 dialog types

---

### C) State Management

**Current state:**
```rust
pub struct AppModel {
    pub dialog: Option<ShowDialog>,  // Current overlay dialog
    ...
}
```

**New state (likely):**
```rust
pub struct AppModel {
    pub dialog_windows: HashMap<WindowId, ShowDialog>,  // Track all open dialogs
    ...
}
```

**Message routing:**
- Window spawn messages: `Message::OpenDialog(ShowDialog)`
- Window close messages: `Message::CloseDialogWindow(WindowId)`
- Dialog-specific messages route to correct window via ID

**Window lifecycle:**
1. User action triggers dialog
2. `spawn_dialog_window(dialog)` creates window
3. Window ID stored in `AppModel`
4. Dialog view renders in window context
5. User interacts (form inputs, buttons)
6. Close via:
   - OK/Cancel/Apply button → explicit close message
   - Window X button → window close event
   - Escape key → mapped to Cancel
7. Window cleanup: remove from tracking, update main view if needed

---

### D) Files Likely Touched

**Core application:**
- `disks-ui/src/ui/app/state.rs` — dialog window tracking
- `disks-ui/src/ui/app/message.rs` — window spawn/close messages
- `disks-ui/src/ui/app/update/mod.rs` — window lifecycle handlers
- `disks-ui/src/ui/app/mod.rs` — Application trait impl (multi-window support)
- `disks-ui/src/ui/app/view.rs` — remove `fn dialog()` override

**Dialog modules:**
- `disks-ui/src/ui/dialogs/view/*.rs` — adapt all 17 dialog types for window context
- `disks-ui/src/ui/dialogs/window.rs` (new) — dialog window view dispatcher
- `disks-ui/src/ui/app/update/dialogs.rs` (new) — dialog window lifecycle helpers

**Tests:**
- Update integration tests if they relied on dialog overlay behavior
- Manual test suite for all dialog types

---

## User/System Flows

### Flow 1: User Formats Partition
1. User clicks "Format" button on a partition
2. **New behavior**: Format dialog opens in a separate window (centered on parent)
3. Parent window is dimmed/disabled (modal state)
4. User fills form (filesystem type, encryption options), clicks "Format"
5. Async operation runs, progress indicator shows in dialog ("Running...")
6. On completion, dialog window closes automatically, parent window refreshes
7. **Alt flow**: User clicks window X button → treated as Cancel, closes dialog

### Flow 2: User Creates Partition
1. User selects free space, clicks "+ Partition"
2. Create partition dialog opens in separate window
3. User selects filesystem type, size, encryption options
4. User clicks "Apply"
5. Operation runs async, dialog remains open with progress indicator
6. On success, dialog closes, main view updates with new partition
7. **Edge case**: If parent window closes while dialog open, dialog also closes (cleanup)

### Flow 3: Multiple Dialogs (Edge Case)
1. User opens "Format" dialog
2. Before closing, user triggers error that shows Info dialog
3. Info dialog opens as second window
4. User closes Info dialog
5. Format dialog remains open
6. User completes format, both dialogs closed
7. **Note**: Prevent certain dialog combinations via app logic (e.g., don't allow second partition dialog)

### Flow 4: Dialog with Nested Async Operations (Image Operations)
1. User clicks "Create Disk Image"
2. Dialog window opens
3. User clicks "Pick Save Location" → COSMIC file picker opens (nested dialog)
4. User selects path, file picker closes
5. User enters size, clicks "Create"
6. Image creation runs with progress bar
7. On completion, dialog closes
8. **Complexity**: File picker dialog within dialog window must work correctly

---

## Risks & Mitigations

### Risk 1: COSMIC Window Management API Limitation
**Risk:** COSMIC may not expose APIs for spawning modal child windows from a single application process.

**Impact:** High — core feature depends on this capability

**Mitigation:**
- Early research phase (Task 1): consult COSMIC docs, libcosmic source, community
- Fallback plan: Use dialog process + IPC (more complex but proven pattern)
- Worst case: Keep dialogs as overlays but improve styling/UX (shadows, better focus management)

**Likelihood:** Low-Medium (COSMIC is built on iced, which supports multi-window apps)

---

### Risk 2: Dialog State Synchronization
**Risk:** With dialogs in separate windows, state updates (e.g., running→success→close) become more complex to coordinate.

**Impact:** Medium — could cause bugs like dialogs not closing, stale data display

**Mitigation:**
- Use message passing pattern (window messages routed to main update loop)
- Window IDs used to track which dialog is active
- Centralized state management in `AppModel` but reference by window ID
- Comprehensive testing of all async dialog operations

**Likelihood:** Medium (complexity inherent in multi-window apps)

---

### Risk 3: Dialog Migration Breaking Existing Workflows
**Risk:** Users accustomed to overlay dialogs may find separate windows disruptive.

**Impact:** Low-Medium — UX change but generally considered an improvement

**Mitigation:**
- Ensure dialogs position sensibly (centered on parent, not arbitrary screen location)
- Keep keyboard shortcuts working (Escape to cancel, Enter to confirm when applicable)
- Thorough testing of all 17 dialog types
- Consider future config option to toggle behavior (out of scope for this spec)

**Likelihood:** Low (separate windows are standard desktop behavior)

---

### Risk 4: Increased Complexity in Update Loop
**Risk:** Routing messages to/from multiple windows adds complexity to the update handler.

**Impact:** Medium — harder to debug, potential for message routing bugs

**Mitigation:**
- Create helper functions for common patterns (spawn, close, route message)
- Document message flow clearly in code comments
- Unit tests for message routing logic (if feasible)
- Incremental migration (one dialog category at a time)

**Likelihood:** Medium-High (expected complexity increase)

---

### Risk 5: Platform-Specific Edge Cases
**Risk:** Window management behavior may differ across Wayland compositors or with X11 fallback.

**Impact:** Low — repo targets COSMIC desktop primarily, but portability matters

**Mitigation:**
- Test on COSMIC desktop (primary target)
- Document any known issues with other compositors
- COSMIC runtime should abstract most platform differences

**Likelihood:** Low (COSMIC runtime handles this)

---

## Acceptance Criteria

### Functional Requirements
- [ ] All 17 dialog types open in separate OS-level windows
- [ ] Dialog windows are modal (parent window is non-interactive while dialog open)
- [ ] Dialog windows can be moved with mouse (title bar drag)
- [ ] Dialogs are centered on parent window by default
- [ ] Window close button (X) functions as Cancel
- [ ] Escape key still closes dialogs
- [ ] Enter key confirms when appropriate (form submit behavior)

### Dialog Functionality Preservation
- [ ] Form validation works as before (size limits, name constraints, password match)
- [ ] Async operation states display correctly (running indicators, success/error)
- [ ] Error messages show in dialogs without premature closure
- [ ] File picker dialogs work within modal windows (nested dialogs)
- [ ] SMART data table displays correctly (scrollable if needed)
- [ ] Unmount busy dialog shows processes and kill actions work

### Window Lifecycle
- [ ] Closing parent window closes all child dialog windows
- [ ] Dialogs maintain proper z-order (stay above parent)
- [ ] Opening multiple info dialogs works (e.g., errors during operations)
- [ ] Canceling a dialog mid-operation properly cleans up (async task cancellation if applicable)

### Code Quality
- [ ] No compiler warnings (`cargo clippy --workspace`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Code follows repo conventions (module structure, error handling)
- [ ] Conventional commit messages for each logical change

### Testing & Validation
- [ ] At least 3 dialog types tested end-to-end (create partition, format, confirm action)
- [ ] Full manual test suite executed for all 17 dialog types
- [ ] No regressions in existing dialog functionality
- [ ] Performance acceptable (dialog spawn time <200ms)

---

## Open Questions

### Q1: COSMIC Multi-Window API
**Question:** Does libcosmic support spawning child windows with parent references? If not, what's the recommended pattern?

**Action:** Research COSMIC docs, ask in cosmic-epoch Matrix/Discord  
**Decision needed by:** Task 1 completion  
**Impact:** Determines entire implementation approach

---

### Q2: Dialog Window Lifecycle
**Question:** Should dialogs survive parent window minimize/hide, or always follow parent state?

**Current thinking:** Dialogs should minimize with parent (standard modal behavior)  
**Decision:** Default to standard modal behavior, can adjust if COSMIC conventions differ

---

### Q3: Multiple Dialogs Policy
**Question:** Should app prevent multiple dialogs of same type (e.g., two "Create Partition" dialogs)?

**Current thinking:** Allow multiple Info/error dialogs, but prevent duplicate operation dialogs  
**Decision:** Implement safeguards in update logic (check if dialog type already open before spawning)

---

### Q4: Dialog Size and Position Persistence
**Question:** Should dialog window positions/sizes be saved across sessions?

**Current thinking:** No, always center on parent (simpler, more predictable)  
**Decision:** Out of scope for initial implementation, revisit in future enhancement

---

## Related Work

- **README V1 Goal #1**: "Dialogs are still ugly" — this spec directly addresses presentation
- **GAP-004** (Dialog State Management): Internal refactoring out of scope, but modal windows may simplify state management long-term
- **Feature: ui-refactor**: Sidebar/navigation changes are separate; dialogs integrate with new UI regardless
- **GNOME Disks**: Reference implementation for modal dialog behavior patterns

---

## Success Metrics

**User-facing improvements:**
- Dialogs feel more "native" and integrated with desktop
- Users can reposition dialogs if main window content is relevant
- Keyboard shortcuts (Escape, Enter) work consistently

**Developer benefits:**
- Clearer separation between main window and dialog state
- Easier to test dialogs in isolation
- Foundation for future dialog enhancements (size persistence, custom positioning)

**No regressions:**
- All existing functionality works identically
- No new crashes or errors
- Performance unchanged or improved
