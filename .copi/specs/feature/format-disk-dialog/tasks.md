# `feature/format-disk-dialog` — Tasks

Status (auto-updated): Implemented

## Task 1: Add dialog state + messages (UI plumbing)

Status: Implemented

- Scope: Add a new dialog state and message flow for “Format Disk”.
- Files/areas:
  - `disks-ui/src/app.rs`
- Steps:
  - Add `ShowDialog::FormatDisk(FormatDiskDialog)` (new struct).
  - Add a message enum for dialog updates/submit (e.g. `FormatDiskMessage`) and a `Message::FormatDisk(FormatDiskMessage)` variant.
  - Ensure selecting `Message::Format` opens the dialog using defaults derived from the active `DriveModel`.
  - Add update handling for combobox changes, cancel, and confirm.
- Test plan:
  - `cargo test --workspace --all-features`
  - `cargo clippy --workspace --all-features`
- Done when:
  - Format Disk no longer shows the “not implemented” info dialog.
  - App can open/close the dialog and mutate state without panicking.

## Task 2: Implement the Format Disk dialog view

Status: Implemented

- Scope: Implement a dialog with two dropdowns and destructive confirm.
- Files/areas:
  - `disks-ui/src/views/dialogs.rs`
- Steps:
  - Add a new dialog constructor (e.g. `dialogs::format_disk(state)`).
  - Use `dropdown` widgets for both comboboxes.
  - Ensure primary action is destructive and disabled/busy while running.
- Test plan:
  - Manual: open Disk → Format Disk and verify options & defaults.
  - `cargo fmt --all --check`
- Done when:
  - Dialog renders with the required combobox options and can dispatch messages.

## Task 3: i18n for dialog labels and option strings

Status: Implemented

- Scope: Add Fluent strings for new UI text.
- Files/areas:
  - `disks-ui/i18n/en/cosmic_ext_disks.ftl`
  - `disks-ui/i18n/sv/cosmic_ext_disks.ftl`
- Steps:
  - Add keys for: “Erase”, “Partitioning”, “Don’t Overwrite (Quick)”, “Overwrite (Slow)”, “Legacy Compatible (DOS/MBR)”, “Modern (GPT)”, “None”.
  - Wire the dialog to use `fl!(...)` keys.
- Test plan:
  - Manual: run app in default locale and verify strings appear.
- Done when:
  - No hard-coded English strings in the new dialog.

## Task 4: Wire confirm → backend operation and refresh

Status: Implemented (best-effort UDisks `Block.Format` mapping; verify `empty` support on target systems)

- Scope: Implement (or stub with clear TODO) the async operation invoked on confirm.
- Files/areas:
  - `disks-ui/src/app.rs`
  - Likely new or extended API in `disks-dbus` (e.g. `disks-dbus/src/disks/drive.rs`)
- Steps:
  - Define request mapping: erase (quick vs overwrite) and partitioning (none/dos/gpt).
  - If `disks-dbus` lacks a whole-disk API, add a `DriveModel` method that:
    - creates/replaces partition table for `dos`/`gpt`
    - handles `none` via the closest supported UDisks action (document final behavior)
    - supports overwrite via UDisks format options or a best-effort equivalent
  - In UI, run the async task and refresh drives/nav on success.
  - On error: show `ShowDialog::Info` with `e`.
- Test plan:
  - Unit tests in `disks-dbus` for mapping/argument building where feasible.
  - Manual (requires permissions and a test drive): verify the expected on-disk result.
- Done when:
  - Confirm runs an async operation and UI refreshes; errors are surfaced.

## Task 5: Quality gates

Status: Implemented

- Scope: Ensure repo-wide quality gates pass.
- Steps:
  - Run `cargo fmt --all --check`.
  - Run `cargo clippy --workspace --all-features`.
  - Run `cargo test --workspace --all-features`.
- Done when:
  - All checks pass cleanly.

## Task 6 (Stretch): Add “working” state to all pre-action dialogs

Status: Implemented

- Scope: Reuse the “working”/busy indicator pattern from Format Disk for other dialogs that immediately precede an async action.
- Dialogs to cover (current codebase):
  - Create Partition (`ShowDialog::AddPartition` → `dialogs::create_partition`)
  - Unlock Encrypted (`ShowDialog::UnlockEncrypted` → `dialogs::unlock_encrypted`)
  - Delete Partition confirmation (`ShowDialog::DeletePartition` → `dialogs::confirmation`)
  - Format Disk is already implemented (`ShowDialog::FormatDisk`)
- Steps:
  - Refactor each dialog state to include a `running`/busy boolean (UI-side wrapper structs; avoid pushing UI-only state into `disks-dbus` data types).
  - Ensure the primary action button disables while running.
  - Add a consistent status line (reuse existing `working` i18n key).
  - Keep the dialog open until the task completes (instead of closing immediately), then close on success/refresh.
- Test plan:
  - Manual: trigger each action and confirm the dialog shows “Working…” and buttons are disabled.
  - `cargo clippy --workspace --all-features` and `cargo test --workspace --all-features`.
- Done when:
  - All pre-action dialogs listed above show a working state and do not allow double-submission.
