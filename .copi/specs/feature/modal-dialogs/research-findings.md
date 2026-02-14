# Modal Dialog Windows — Research Findings & Updated Plan

**Date:** 2026-02-12  
**Branch:** `feature/modal-dialogs`  
**Research Status:** Task 1 Complete

---

## Executive Summary

After researching COSMIC/libcosmic window management APIs, we've discovered a **critical limitation**: libcosmic does not currently support true modal child windows with parent-child relationships). All windows opened via `window::open()` are independent top-level windows.

This significantly impacts the feasibility of the original spec goal: converting all dialogs to "OS-level modal windows."

---

## Research Findings

### Multi-Window Support Status

✅ **Available:**
- `window::open(settings)` — creates new independent windows
- `window::close(id)` — closes windows
- `Application::view_window(id)` — renders content for additional windows
- `on_close_requested(id)` — handles close events
- Window tracking via `window::Id`

❌ **NOT Available:**
- Parent-child window relationships (transient_for)
- Modal window semantics (window blocker behavior)
- Window parenting APIs

**Evidence:** libcosmic source code comment in `get_window()`:
```rust
None, // TODO parent for window, platform specific option maybe?  
```

This is a TODO item — the feature doesn't exist yet.

### Current Dialog Approach

The current implementation uses:
```rust
fn dialog(&self) -> Option<Element<'_, Self::Message>>
```

This returns widgets rendered as **overlay elements** using `cosmic::widget::popover` with `modal(true)`, centered over the main window. This approach:
- ✅ Works reliably
- ✅ Has proper modal behavior (blocks interaction with underlying content)
- ✅ Is integrated into the COSMIC framework
- ❌ Renders within the window, not as separate OS windows
- ❌ Cannot be moved independently
- ❌ Cannot be minimized/maximized separately

### Wayland Popup Surfaces

libcosmic extensively supports Wayland `xdg_popup` surfaces (for menus, dropdowns, tooltips):
- `SctkPopupSettings` with parent/anchor/gravity
- Parent-child relationships via `popup::get_popup()`
- These are **NOT** independent windows — they're ephemeral surfaces attached to a parent

---

## Implications for Spec

### Original Goal
"Convert all 17 dialog types from overlay widgets to separate OS-level modal windows"

### Feasibility Assessment

**Option A: True Modal Windows** ❌ **NOT FEASIBLE**  
- Requires parent-child window relationships
- Not supported by libcosmic yet
- Would require upstream contribution to libcosmic/iced

**Option B: Independent Top-Level Windows** ⚠️ **POSSIBLE BUT PROBLEMATIC**  
- Can create using `window::open()`
- Windows would be **independent**, not modal children
- Issues:
  - No automatic position relative to parent
  - No z-order enforcement (could go behind main window)
  - No automatic close-on-parent-close behavior
  - Confusing UX (dialogs appear as separate app instances in taskbar/alt-tab)
  - User could interact with main window while dialog is open (unless we manually block)

**Option C: Enhanced Overlay System** ✅ **RECOMMENDED**  
- Keep using `Application::dialog()` overlay approach
- Improve the rendering/styling of dialogs to look better
- Possibly add features like:
  - Draggable dialog headers (within window bounds)
  - Better backdrop dimming
  - Smoother animations
  - Improved sizing/positioning

**Option D: Wayland Layer Shell** 🤔 **UNCERTAIN**  
- COSMIC layer shell could theoretically create overlay surfaces
- Would still not be "windows" in the traditional sense
- Requires investigation if layer shell supports this use case

---

## Updated Recommendation

Given the research findings, I recommend **pivoting the spec** to one of:

### 1. "Wait and Contribute Upstream" (deferred)
- File issue/RFC with libcosmic team
- Implement parent-child window relationships in iced/libcosmic
- Return to this feature once upstream support exists
- **Timeline:** Months to years

### 2. "Enhanced Overlay Dialogs" (practical)
- **Goal:** Significantly improve dialog UX within existing overlay system
- Keep using `Application::dialog()` but make dialogs:
  1. Draggable (via header bar)
  2. Escapable (Esc key closes)
  3. Better styled (proper elevation, shadows, borders)
  4. Smooth animations (fade in/out)
  5. Better keyboard navigation (tab order, focus management)
  6. Proper accessibility (screen reader announcements)
- **Timeline:** 15-25 hours (much less than original 30-49h)
- **Benefit:** Addresses V1 goal #1 "Dialogs are still ugly" without architectural constraints

### 3. "Hybrid Approach" (experimental)
- Use enhanced overlays for most dialogs
- Use independent top-level windows ONLY for:
  - Long-running operations (e.g., imaging dialog)
  - Dialogs that benefit from independence (SMART data viewer)
- Accept UX compromises for those specific cases
- **Timeline:** 25-35 hours

---

## Questions for Product Decision

1. **What is the primary pain point with current dialogs?**
   - Visual appearance? (shadows, borders, spacing)
   - Interaction? (can't move them, can't see underlying content)
   - Functionality? (specific dialog is problematic)

2. **What is the expected behavior for "modal dialog windows"?**
   - Must be separate OS windows? (not currently possible)
   - OR can be improved overlays? (achievable now)

3. **Priority:**
   - Ship improved UX soon (enhanced overlays)
   - Wait for perfect solution (true modal windows, long timeline)

4. **Is this blocking V1 release?**
   - If yes, recommend Option 2 (Enhanced Overlays)
   - If no, recommend Option 1 (Wait for Upstream)

---

## Proposed Next Steps

**Immediate:**
1. ✅ Complete research (this document)
2. ⏳ Get product/user feedback on above questions
3. ⏳ Decide on approach (Option 1, 2, or 3)

**If Option 2 chosen (Enhanced Overlays):**
1. Update [plan.md](plan.md) with revised scope
2. Rewrite [tasks.md](tasks.md) focusing on overlay improvements
3. Create proof-of-concept with draggable dialog
4. Iterate on visual design

**If Option 1 chosen (Upstream Contribution):**
1. Research iced window management architecture
2. Design API for parent-child window relationships
3. Create RFC for iced community
4. Implement in libcosmic fork
5. Submit upstream PR
6. Wait for acceptance and COSMIC integration
7. Return to this feature (6+ months timeline)

**If Option 3 chosen (Hybrid):**
1. Update specs for split approach
2. Start with enhanced overlays for most dialogs
3. Prototype independent windows for 2-3 specific cases
4. Evaluate UX tradeoffs

---

## Technical Notes

### Window Opening Pattern (for reference)
```rust
// How to open an independent window (if we go that route)
let (id, spawn_task) = window::open(window::Settings {
    position: window::Position::Default,
    exit_on_close_request: false,  // Don't exit app when dialog closes
    decorations: true,              // Show title bar
    resizable: false,               // Dialogs typically non-resizable
    ..Default::default()
});

// Then render it via view_window()
fn view_window(&self, id: window::Id) -> Element<'_, Self::Message> {
    match self.windows.get(&id) {
        Some(WindowKind::Dialog(dialog)) => render_dialog(dialog),
        _ => panic!("Unknown window"),
    }
}
```

### Current Dialog Pattern
```rust
// Current approach (overlay)
fn dialog(&self) -> Option<Element<'_, Self::Message>> {
    match self.dialog_state {
        Some(ShowDialog::Info { title, body }) => {
            Some(dialogs::info(title, body, Message::CloseDialog))
        }
        None => None,
    }
}
```

### libcosmic Revision
Using git rev: `beddbf17703728182395a13267954d839226331d`

---

## References

- [libcosmic multi-window example](https://github.com/pop-os/libcosmic/tree/main/examples/multi-window)
- [iced window management](https://docs.rs/iced/latest/iced/window/index.html)
- [Current dialog implementation](./../../../storage-ui/src/ui/app/view.rs#L28-119)
- [Wayland xdg_popup protocol](https://wayland.app/protocols/xdg-shell#xdg_popup)
