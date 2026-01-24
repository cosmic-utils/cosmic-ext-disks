# Spec Index

A lightweight mapping of audited gaps/work items to implementation specs.

| Gap ID | Title | Spec Path | Branch | Source Audit | Status |
|---|---|---|---|---|---|
| GAP-001 | Create-partition dialog “Cancel” crashes the app | `.copi/specs/fix/prevent-ui-panics/` | `fix/prevent-ui-panics` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented |
| GAP-002 | Create-partition dialog can panic on unexpected state | `.copi/specs/fix/prevent-ui-panics/` | `fix/prevent-ui-panics` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented |
| GAP-003 | Several menu actions are wired to `todo!()` (crash-on-click) | `.copi/specs/fix/prevent-ui-panics/` | `fix/prevent-ui-panics` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented |
| GAP-004 | Partition segmentation uses hacks (offset and trailing bytes) | `.copi/specs/fix/partition-segmentation-hacks/` | `fix/partition-segmentation-hacks` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented (manual validation pending) |
| GAP-004 | GPT reserved offsets show as free space (UDisks rejects creation) | `.copi/specs/fix/gpt-reserved-offsets-udisks/` | `fix/gpt-reserved-offsets-udisks` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented (manual validation pending) |
| GAP-005 | MBR/DOS partition creation likely broken (table type mismatch) | `.copi/specs/fix/gap-005-dos-msdos-table-type/` | `fix/gap-005-dos-msdos-table-type` | `.copi/audits/2026-01-24T18-03-04Z.md` | Implemented (manual validation pending) |
| GAP-008 | Device change detection is polling-based (1s) instead of signal-based | `.copi/specs/fix/device-change-detection-signals/` | `fix/device-change-detection-signals` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented |
| N/A | Remove old/backup polling for device events logic | `.copi/specs/fix/remove-device-polling-fallback/` | `fix/remove-device-polling-fallback` | N/A (brief) | Implemented |
