# Spec Index

A lightweight mapping of audited gaps/work items to implementation specs.

| Gap ID | Title | Spec Path | Branch | Source Audit | Status |
|---|---|---|---|---|---|
| GAP-001 | Create-partition dialog “Cancel” crashes the app | `.copi/specs/fix/prevent-ui-panics/` | `fix/prevent-ui-panics` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented |
| GAP-002 | Create-partition dialog can panic on unexpected state | `.copi/specs/fix/prevent-ui-panics/` | `fix/prevent-ui-panics` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented |
| GAP-003 | Several menu actions are wired to `todo!()` (crash-on-click) | `.copi/specs/fix/prevent-ui-panics/` | `fix/prevent-ui-panics` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented |
| GAP-004 | Partition segmentation uses hacks (offset and trailing bytes) | `.copi/specs/fix/partition-segmentation-hacks/` | `fix/partition-segmentation-hacks` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented (manual validation pending) |
| GAP-004 | GPT reserved offsets show as free space (UDisks rejects creation) | `.copi/specs/fix/gpt-reserved-offsets-udisks/` | `fix/gpt-reserved-offsets-udisks` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented (manual validation pending) |
| GAP-005 | MBR/DOS partition creation likely broken (table type mismatch) | `.copi/specs/fix/gap-005-dos-msdos-table-type/` | `fix/gap-005-dos-msdos-table-type` | `.copi/audits/2026-01-24T18-03-04Z.md` | Implemented |
| GAP-007 | Mount state detection relies on parsing `df` | `.copi/specs/fix/gap-007-mount-state-detection/` | `fix/gap-007-mount-state-detection` | `.copi/audits/2026-01-24T18-03-04Z.md` | Implemented |
| GAP-008 | Device change detection is polling-based (1s) instead of signal-based | `.copi/specs/fix/device-change-detection-signals/` | `fix/device-change-detection-signals` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented |
| GAP-010 | Test coverage is incomplete for destructive/system-integrated flows | `.copi/specs/chore/gap-010-012-tests-release-spdx/` | `chore/gap-010-012-tests-release-spdx` | `.copi/audits/2026-01-24T18-03-04Z.md` | Implemented |
| GAP-011 | Release pipeline publishes with `--allow-dirty --no-verify` | `.copi/specs/chore/gap-010-012-tests-release-spdx/` | `chore/gap-010-012-tests-release-spdx` | `.copi/audits/2026-01-24T18-03-04Z.md` | Implemented |
| GAP-012 | SPDX header placeholder in i18n module | `.copi/specs/chore/gap-010-012-tests-release-spdx/` | `chore/gap-010-012-tests-release-spdx` | `.copi/audits/2026-01-24T18-03-04Z.md` | Implemented |
| N/A | Remove old/backup polling for device events logic | `.copi/specs/fix/remove-device-polling-fallback/` | `fix/remove-device-polling-fallback` | N/A (brief) | Implemented |
| N/A | Support LUKS + logical/nested volumes (unlock prompt + show inner filesystems) | `.copi/specs/feature/luks-logical-volumes/` | `feature/luks-logical-volumes` | N/A (brief) | Implemented |
| N/A | LUKS delete preflight (unmount children + lock) + hide delete for child volumes | `.copi/specs/fix/luks-delete-preflight/` | `fix/luks-delete-preflight` | N/A (brief; PR #36 follow-up) | Implemented |
| N/A | Create-partition “Password protected” should create LUKS | `.copi/specs/fix/create-partition-password-protection/` | `fix/create-partition-password-protection` | N/A (brief; 2026-02-04) | Implemented |
| N/A | Write logs to file + log UI surfaced errors + preserve UDisks2 method error context | `.copi/specs/fix/file-logging-ui-error-context/` | `fix/file-logging-ui-error-context` | N/A (brief; 2026-02-04) | Implemented |
| N/A | UI refactor: replace sidepanel navigation for rich items + sections | `.copi/specs/feature/ui-refactor/` | `feature/ui-refactor` | N/A (brief; 2026-02-06) | In Progress (Phase 4) |
| N/A | Disk → Format Disk dialog (erase + partitioning) | `.copi/specs/feature/format-disk-dialog/` | `feature/format-disk-dialog` | N/A (brief) | Implemented |
| N/A | Implement remaining Disk menu commands (SMART/power/standby) | `.copi/specs/feature/disk-menu-commands/` | `feature/disk-menu-commands` | N/A (brief) | Spec created |
| N/A | Implement all Image menu commands (disk imaging + attach/new image) | `.copi/specs/feature/image-menu-commands/` | `feature/image-menu-commands` | N/A (brief) | Implemented |
| N/A | Implement all volume commands (actionbar + dialogs + DBus) | `.copi/specs/feature/volume-commands-actionbar/` | `feature/volume-commands-actionbar` | N/A (brief; 2026-01-25) | Implemented |
| N/A | GNOME Disks parity: Edit Mount Options + Edit Encryption Options dialogs | `.copi/specs/feature/volume-commands-actionbar/` | `feature/volume-commands-actionbar` | N/A (brief; 2026-01-25 addendum) | Implemented |
| N/A | Architecture cleanup (abstractions/naming/hierarchy) — covers audit 2026-01-25 structural gaps | `.copi/specs/chore/audit-2026-01-25-architecture-cleanup/` | `chore/audit-2026-01-25-architecture-cleanup` | `.copi/audits/2026-01-25T23-24-44Z.md` | Implemented |
| N/A | Unmount resource busy error recovery with procfs-based process detection and kill options | `.copi/specs/feature/unmount-busy-error-recovery/` | `feature/unmount-busy-error-recovery` | N/A (brief; 2026-02-11) | Spec created |
| N/A | Filesystem tools detection and status display in settings pane (with localization) | `.copi/specs/feature/filesystem-tools-detection/` | `main` | N/A (brief; 2026-02-11) | Implemented |
| N/A | Improve Partitioning view UI/UX (conditional name field, unit-aware size inputs, clearer labels, radio list for filesystem types with tool detection) | `.copi/specs/feature/improve-partitioning-view/` | `feature/improve-partitioning-view` | N/A (brief; 2026-02-11) | Spec created |
