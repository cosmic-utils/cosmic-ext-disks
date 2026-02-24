# Usage UI Tweaks Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement targeted Usage tab UX refinements for loading, wizard, and result layout while extracting shared wizard generics based on the existing rclone wizard as canonical baseline.

**Architecture:** Extract shared wizard presentation primitives from current network/rclone wizard UI and recompose both network and usage wizard views on top. Then apply usage-specific layout tweaks in `usage_tab_view` for wrapping tabs, action-bar alignment, and fill-height file list scrolling.

**Tech Stack:** Rust (`storage-ui` crate), COSMIC/iced widgets/layout, existing app message/update architecture.

---

## Task 1: Extract shared wizard UI generics from rclone wizard

**Files:**
- Add: `storage-ui/src/ui/...` shared wizard module files (exact location chosen to match existing module organization)
- Modify: `storage-ui/src/ui/network/view.rs`

**Steps:**
1. Create shared presentation primitives:
   - `WizardShell`
   - `WizardActionRow`
   - `OptionTileGrid`
   - `SelectableTile` (selected/unselected style behavior)
2. Copy sizing/spacing/card geometry from current rclone wizard as baseline.
3. Refactor network/rclone wizard view to use the shared primitives without changing step logic/state transitions.

**Validation:**
- `cargo check -p cosmic-ext-storage`

---

## Task 2: Rebuild usage wizard on shared generics

**Files:**
- Modify: `storage-ui/src/ui/app/view.rs`

**Steps:**
1. Replace usage wizard layout with `WizardShell` composition matching rclone spacing/sizing.
2. Replace mount-point checkboxes with wrapping toggle tiles using shared `OptionTileGrid` + `SelectableTile`.
3. Preserve existing messages and validation behavior:
   - mount loading/empty states
   - show-all toggle
   - parallelism dropdown
   - start enabled conditions
   - cancel behavior

**Validation:**
- `cargo check -p cosmic-ext-storage`
- Manual: wizard layout visually aligned with rclone wizard.

---

## Task 3: Update usage loading screen layout

**Files:**
- Modify: `storage-ui/src/ui/app/view.rs`

**Steps:**
1. Center loading content vertically and horizontally in usage area.
2. Render label + progress bytes on one row with right-aligned bytes text.
3. Constrain progress bar width with sensible responsive max width.

**Validation:**
- `cargo check -p cosmic-ext-storage`
- Manual: large window shows centered content and non-full-width progress bar.

---

## Task 4: Rework usage category tabs and action bar layout

**Files:**
- Modify: `storage-ui/src/ui/app/view.rs`

**Steps:**
1. Replace fixed `chunks(3)` category-tab rows with true wrapping layout.
2. Keep category tabs directly below segmented usage bar.
3. Reorder action bar:
   - left: Number of files, Refresh
   - right: Selected count, Clear Selection, Delete
4. Apply action-button styling for Refresh/Clear/Delete consistent with drive header/volume-control actions.

**Validation:**
- `cargo check -p cosmic-ext-storage`
- Manual: tab wrapping responds to width; action-bar ordering/alignment matches spec.

---

## Task 5: Make file list fill remaining height with contained scrolling

**Files:**
- Modify: `storage-ui/src/ui/app/view.rs`

**Steps:**
1. Restructure usage content container heights to fill parent.
2. Ensure only file-list region scrolls for large file sets (avoid full-page scroll behavior for this view).
3. Update typography:
   - file header row normal-size bold
   - file rows normal-size text

**Validation:**
- `cargo check -p cosmic-ext-storage`
- Manual: list expands to available height and scrolls independently.

---

## Task 6: Verification sweep and polish

**Files:**
- Modify as needed for compile/test fixes scoped to touched UI files

**Steps:**
1. Run targeted compile/tests:
   - `cargo check -p cosmic-ext-storage`
   - any targeted UI tests near touched helpers/modules
2. Manual smoke checks against acceptance criteria in design doc:
   - loading alignment
   - wizard visual parity with rclone baseline
   - mount toggle tiles
   - wrapping tabs
   - action bar order/alignment
   - fill-height list + contained scroll
   - typography sizing

**Validation:**
- All checks pass; manual criteria met.

---

## Notes
- Keep scope to requested UX changes only.
- Do not alter service/API behavior for this task.
- Prefer smallest possible refactor that still delivers shared wizard generics and visual coherence.