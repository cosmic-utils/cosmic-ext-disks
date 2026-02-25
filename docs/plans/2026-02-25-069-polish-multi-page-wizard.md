# 069 Polish Multi-Page Wizard Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Convert specified complex storage wizards to shared breadcrumb-driven multi-page flows with strict per-step validation and no unrelated visual restyling.

**Architecture:** Introduce shared wizard step primitives in `ui/wizard.rs`, then migrate each target dialog to explicit step state and shared navigation semantics. Keep final submit operations unchanged and reuse existing backend message paths.

**Tech Stack:** Rust, libcosmic/iced UI, cargo fmt/check/clippy.

---

## Requirement Checklist
- [ ] R1 Shared breadcrumb primitive used across all in-scope wizards.
- [ ] R2 Shared step nav primitive used across all in-scope wizards.
- [ ] R3 Create partition uses 3 pages in approved order.
- [ ] R4 Next button is blocked until current step is valid.
- [ ] R5 Breadcrumb clicks only navigate to previous steps.
- [ ] R6 No unrelated wizard styling changes.

## Task 1: Add shared step primitives

**Files:**
- Modify: `storage-ui/src/ui/wizard.rs`

**Step 1: Add failing tests for breadcrumb navigation policy**
- Add unit tests for helper logic that determines whether a step is clickable.
- Cases: previous step allowed, current disabled, future disabled.

**Step 2: Run targeted test to verify failure**
- Run: `cargo test -p cosmic-ext-storage wizard::tests::breadcrumb_*`
- Expected: helper tests fail until new helper functions exist.

**Step 3: Implement minimal shared helpers and views**
- Add `wizard_breadcrumb`, `wizard_step_nav`, `wizard_step_shell`.
- Add helper functions for `is_step_clickable` and `is_next_enabled`.

**Step 4: Re-run targeted tests**
- Run: `cargo test -p cosmic-ext-storage wizard::tests::breadcrumb_*`
- Expected: pass.

**Step 5: Commit**
- `git add storage-ui/src/ui/wizard.rs`
- `git commit -m "ui: add shared wizard step primitives"`

## Task 2: Add create-partition step state/messages

**Files:**
- Modify: `storage-ui/src/ui/dialogs/state.rs`
- Modify: `storage-ui/src/ui/dialogs/message.rs`
- Modify: `storage-ui/src/ui/volumes/update/create.rs`
- Modify: `storage-ui/src/ui/app/view.rs`

**Step 1: Add failing tests for step transitions**
- Add tests covering initial step and `PrevStep/NextStep` transitions.

**Step 2: Run tests and confirm red**
- Run: `cargo test -p cosmic-ext-storage create_partition_step_*`
- Expected: fail before transition logic exists.

**Step 3: Implement step enum + transition handling**
- Add `CreatePartitionStep` enum.
- Add `PrevStep`/`NextStep` message handling with bounds.
- Initialize new dialogs at first step.

**Step 4: Re-run tests and confirm green**
- Run same targeted test command.

**Step 5: Commit**
- `git add storage-ui/src/ui/dialogs/state.rs storage-ui/src/ui/dialogs/message.rs storage-ui/src/ui/volumes/update/create.rs storage-ui/src/ui/app/view.rs`
- `git commit -m "ui: add create partition step state and navigation"`

## Task 3: Refactor create-partition into 3 pages

**Files:**
- Modify: `storage-ui/src/ui/dialogs/view/partition.rs`

**Step 1: Add failing tests for per-step validation helpers**
- Cover: basics valid/invalid, sizing valid/invalid, options password rules.

**Step 2: Run tests and confirm red**
- Run: `cargo test -p cosmic-ext-storage partition_wizard_validation_*`

**Step 3: Implement step content split**
- Step 1: name + filesystem.
- Step 2: sizing controls.
- Step 3: overwrite + password controls.
- Use shared breadcrumb and shared step nav.

**Step 4: Wire final apply only on step 3**
- Keep existing `CreateMessage::Partition` operation flow.

**Step 5: Re-run targeted tests**
- Run same validation tests.

**Step 6: Commit**
- `git add storage-ui/src/ui/dialogs/view/partition.rs`
- `git commit -m "ui: convert create partition to three-step wizard"`

## Task 4: Convert format/edit/resize partition to multi-page

**Files:**
- Modify: `storage-ui/src/ui/dialogs/view/partition.rs`
- Modify (if required): `storage-ui/src/ui/volumes/update/create.rs`

**Step 1: Add failing tests for each flow’s step gating helper**
- Format: basics -> options.
- Edit: type/name -> flags -> review.
- Resize: sizing -> confirm.

**Step 2: Run tests and confirm red**
- Run: `cargo test -p cosmic-ext-storage partition_multi_step_*`

**Step 3: Implement per-flow step enums and content partitioning**
- Use shared breadcrumb/nav components.
- Keep existing submit messages unchanged.

**Step 4: Re-run tests and confirm green**
- Run same targeted tests.

**Step 5: Commit**
- `git add storage-ui/src/ui/dialogs/view/partition.rs storage-ui/src/ui/volumes/update/create.rs`
- `git commit -m "ui: convert partition edit/format/resize to multi-step wizards"`

## Task 5: Convert mount + encryption options to multi-page

**Files:**
- Modify: `storage-ui/src/ui/dialogs/view/mount.rs`
- Modify: `storage-ui/src/ui/dialogs/view/encryption.rs`
- Modify (if required): corresponding update handlers

**Step 1: Add failing tests for next-button gating on required fields**
- Mount options and encryption options validation cases.

**Step 2: Run tests and confirm red**
- Run: `cargo test -p cosmic-ext-storage mount_encryption_wizard_*`

**Step 3: Implement page grouping and breadcrumb navigation**
- Defaults/behavior -> core options -> security/review.
- Previous-step breadcrumb clicks only.

**Step 4: Re-run tests and confirm green**
- Run same targeted tests.

**Step 5: Commit**
- `git add storage-ui/src/ui/dialogs/view/mount.rs storage-ui/src/ui/dialogs/view/encryption.rs`
- `git commit -m "ui: convert mount and encryption options to multi-step wizards"`

## Task 6: Verify and stabilize

**Files:**
- Modify only if required by verification failures.

**Step 1: Format**
- Run: `cargo fmt --all`

**Step 2: UI compile**
- Run: `cargo check -p cosmic-ext-storage`

**Step 3: UI lint strict**
- Run: `cargo clippy -p cosmic-ext-storage -- -D warnings`

**Step 4: Workspace compile**
- Run: `cargo check --workspace`

**Step 5: Requirement pass/fail review**
- Confirm R1-R6 are complete and document any deltas.

**Step 6: Final commit (if needed)**
- `git add -A`
- `git commit -m "ui: finalize multi-page wizard migration"`
