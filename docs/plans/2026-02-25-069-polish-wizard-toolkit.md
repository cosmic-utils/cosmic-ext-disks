# 069 Polish Wizard Toolkit Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Correct all reported polish regressions by introducing a generalized full-page wizard toolkit for non-trivial flows and applying the exact UX fixes requested.

**Architecture:** Implement a single app-level full-page wizard host plus reusable primitives, then migrate non-trivial flows through adapters while retaining simple modal dialogs. Apply sidebar/network/settings fixes in parallel slices, with strict requirement-to-task traceability.

**Tech Stack:** Rust workspace, COSMIC/libcosmic UI, iced widgets, workspace cargo features, cargo fmt, cargo clippy.

---

## Requirement Traceability Checklist (MUST COMPLETE)
- [ ] R1 Image create/attach matches Network wizard styling.
- [ ] R2 Images action row is bottom-most in sidebar.
- [ ] R3 Google Drive + OneDrive logos visible.
- [ ] R4 Amazon S3 + Backblaze + Proton Drive logos visible.
- [ ] R5 Wizard provider-grid caption text centered.
- [ ] R6 Format partition is full-page wizard (non-modal).
- [ ] R7 GitHub icon visible on settings page.
- [ ] R8 App icon/title/tagline removed from settings page.
- [ ] R9 Settings shown first and grouped by domain.
- [ ] R10 Git commit details are caption-sized text.

## Task Checklist (Mapped 1:1)

### Task 1: Generalized wizard host + toolkit scaffold
**Files:**
- Create/Modify: `storage-ui/src/ui/wizard.rs`
- Modify: `storage-ui/src/ui/app/state.rs`
- Modify: `storage-ui/src/ui/app/message.rs`
- Modify: `storage-ui/src/ui/app/view.rs`
- Modify: `storage-ui/src/ui/app/update/mod.rs`

**Steps:**
1. Add app-level full-page wizard host state.
2. Add generalized toolkit contracts (header/body/footer, validation slot, error slot, progress slot).
3. Route non-trivial flow entry points to host state.
4. Keep simple dialog path unchanged.

**Verification:**
- Run: `cargo check -p cosmic-ext-storage`
- Expected: builds with wizard host branch enabled.

### Task 2: Migrate image flows to full-page host (R1)
**Files:**
- Modify: `storage-ui/src/ui/dialogs/view/image.rs` (remove modal wrapper usage for migrated flows)
- Modify: `storage-ui/src/ui/app/view.rs`
- Modify: `storage-ui/src/ui/app/update/image/dialogs.rs`
- Modify: `storage-ui/src/ui/app/update/mod.rs`

**Steps:**
1. Route `NewDiskImage`, `AttachDiskImage`, `ImageOperation` into full-page host.
2. Preserve existing field/progress/error behavior.
3. Ensure footer/actions match Network wizard style.

**Verification:**
- [ ] Mark R1 complete.
- Run: `cargo check -p cosmic-ext-storage`

### Task 3: Migrate partition/disk/btrfs non-trivial flows (R6)
**Files:**
- Modify: `storage-ui/src/ui/dialogs/view/partition.rs`
- Modify: `storage-ui/src/ui/dialogs/view/disk.rs`
- Modify: `storage-ui/src/ui/dialogs/view/btrfs.rs`
- Modify: `storage-ui/src/ui/app/view.rs`
- Modify: `storage-ui/src/ui/app/update/mod.rs`

**Steps:**
1. Move `create/format/edit/resize partition`, `format disk`, `btrfs create/snapshot` to full-page host.
2. Keep `edit filesystem label` and confirmations modal.

**Verification:**
- [ ] Mark R6 complete.
- Run: `cargo check -p cosmic-ext-storage`

### Task 4: Sidebar ordering + images placement (R2)
**Files:**
- Modify: `storage-ui/src/ui/sidebar/view.rs`

**Steps:**
1. Render network section before appending images action row.
2. Ensure images action row is last visual element in sidebar.

**Verification:**
- [ ] Mark R2 complete.
- Manual check: sidebar bottom shows image actions after all sections.

### Task 5: Provider branding visibility + caption centering (R3, R4, R5)
**Files:**
- Modify: `storage-ui/src/ui/network/icons.rs`
- Modify: `storage-ui/src/ui/network/state.rs`
- Modify: `storage-ui/src/ui/network/view.rs`
- Optional assets: `storage-ui/resources/icons/**` (if needed for fallback)

**Steps:**
1. Add explicit mappings for `drive`, `onedrive`, `s3`, `b2`, `protondrive`.
2. Enforce fallback chain to prevent invisible logos.
3. Center provider caption text alignment in wizard grid tiles.

**Verification:**
- [ ] Mark R3 complete.
- [ ] Mark R4 complete.
- [ ] Mark R5 complete.
- Run: `cargo check -p cosmic-ext-storage`

### Task 6: Settings layout/domain grouping + metadata polish (R7, R8, R9, R10)
**Files:**
- Modify: `storage-ui/src/views/settings.rs`
- Modify: `storage-ui/src/ui/app/view.rs` (if ordering/host behavior requires)
- Modify: `storage-ui/i18n/**` (if labels/sectioning strings needed)

**Steps:**
1. Remove app icon/title/tagline block.
2. Put settings controls first and grouped by domain (e.g., scanning, display, integrations/about).
3. Ensure GitHub icon renders at bottom-right.
4. Render commit metadata caption-sized to left of GitHub icon.

**Verification:**
- [ ] Mark R7 complete.
- [ ] Mark R8 complete.
- [ ] Mark R9 complete.
- [ ] Mark R10 complete.
- Run: `cargo check -p cosmic-ext-storage`

### Task 7: Full verification sweep + PR readiness
**Files:**
- Modify only if fixes are required from checks

**Steps:**
1. Run workspace formatting.
2. Run workspace clippy strict.
3. Run workspace build checks.
4. Re-check requirement traceability checklist.

**Commands:**
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check --workspace`

**Exit Criteria:**
- All commands pass.
- Requirement checklist has all items checked.
- No non-trivial flow remains modal.

## Verification Matrix (Requirements -> Tasks)
- R1 -> Task 2
- R2 -> Task 4
- R3 -> Task 5
- R4 -> Task 5
- R5 -> Task 5
- R6 -> Task 3
- R7 -> Task 6
- R8 -> Task 6
- R9 -> Task 6
- R10 -> Task 6
