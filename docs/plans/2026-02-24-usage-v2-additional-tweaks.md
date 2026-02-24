# Usage V2 Additional Tweaks Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement approved Usage V2 tweaks for bar/chip layout, filter behavior, icons, refresh/configure flow, and i18n updates while keeping advanced distro-aware category expansion in design-only scope.

**Architecture:** Update Usage UI state/view/update flow for multi-filter chips and refresh/configure split. Keep restricted-category enforcement in service response contract and render only non-zero categories in UI.

**Tech Stack:** Rust workspace (`storage-ui`, `storage-service`, `storage-sys`, `storage-common`), COSMIC/iced UI, existing D-Bus scan flow.

---

## Task 1: Usage bar height + chip ordering baseline

**Files:**
- Modify: `storage-ui/src/ui/app/view.rs`

**Steps:**
1. Increase usage segmented bar height from 18px to 36px.
2. Build category chip source list from scan result totals sorted descending by bytes.
3. Render only non-zero categories as chip candidates.

**Validation:**
- `cargo check -p cosmic-ext-storage`

---

## Task 2: Convert category tabs to multi-filter chips

**Files:**
- Modify: `storage-ui/src/ui/volumes/state.rs`
- Modify: `storage-ui/src/ui/app/message.rs`
- Modify: `storage-ui/src/ui/app/update/mod.rs`
- Modify: `storage-ui/src/ui/app/view.rs`

**Steps:**
1. Replace single selected category state with selected category set.
2. Add message(s) for toggling filter chips.
3. Default selection to all visible categories after scan load.
4. Enforce non-empty selected set in reducer.
5. Update file list source to combine files across selected categories.

**Validation:**
- `cargo check -p cosmic-ext-storage`
- Add/adjust targeted reducer tests for multi-filter behavior.

---

## Task 3: Add category icons (chips + file list first column)

**Files:**
- Modify: `storage-ui/src/ui/app/view.rs`

**Steps:**
1. Define category-to-icon mapping helper.
2. Render icon in each category chip.
3. Add first-column colored category icon in file list rows.
4. Keep existing selection and row interactions unchanged.

**Validation:**
- `cargo check -p cosmic-ext-storage`

---

## Task 4: Refresh/configure split and wording updates

**Files:**
- Modify: `storage-ui/src/ui/app/message.rs`
- Modify: `storage-ui/src/ui/app/update/mod.rs`
- Modify: `storage-ui/src/ui/app/view.rs`

**Steps:**
1. Make `Refresh` rerun scan with current confirmed config (no wizard reopen).
2. Add `Configure` action to reopen wizard.
3. Rename `Number of files` label to `Files per Category`.
4. Rename toggle label to `Show All Files (Root Mode)`.

**Validation:**
- `cargo check -p cosmic-ext-storage`
- Reducer checks for refresh/configure behavior.

---

## Task 5: Service/UI contract for restricted categories

**Files:**
- Modify: `storage-service` usage scan categorization path(s)
- Modify: `storage-sys` usage classification/category plumbing as needed
- Modify: `storage-ui/src/ui/app/view.rs` (render logic)

**Steps:**
1. Ensure service returns zero/no restricted category results when privilege scope is insufficient.
2. Keep UI policy-free regarding mode; render categories from returned non-zero totals.
3. Validate root/non-root behavior with current ACL probing/auth flows.

**Validation:**
- `cargo check -p storage-service -p cosmic-ext-storage`
- Add focused service test(s) for restricted-category suppression.

---

## Task 6: i18n sweep for surfaced strings

**Files:**
- Modify: `storage-ui/src/ui/**` and `storage-ui/src/views/**` touched by usage flow
- Modify: `storage-ui/i18n/en/cosmic_ext_storage.ftl`
- Modify: additional locale files as required by repo policy

**Steps:**
1. Replace hardcoded newly touched Usage/Wizard strings with `fl!` keys.
2. Add keys for chip labels/tooltips, configure action, renamed labels, and root-mode label.
3. Run search sweep for user-facing literals in affected usage UI paths.

**Validation:**
- `cargo check -p cosmic-ext-storage`

---

## Task 7: Final verification

**Steps:**
1. Run formatting and lint/test checks:
   - `cargo fmt --all`
   - `cargo clippy --workspace --all-targets`
   - targeted usage tests in `storage-ui`
2. Manual smoke:
   - bar height doubled,
   - chips sorted descending and multi-filtering,
   - icons in chips and list rows,
   - refresh/configure behavior,
   - non-root restricted categories absent due to service result,
   - updated labels and i18n text render.

---

## Notes
- Distro-aware expanded category engine remains follow-up work (design complete, implementation deferred).
- Keep changes scoped; avoid adding unrelated UX features.