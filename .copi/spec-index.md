# Spec Index

A lightweight mapping of audited gaps/work items to implementation specs.

| Gap ID | Title | Spec Path | Branch | Source Audit | Status |
|---|---|---|---|---|---|
| GAP-001 | Create-partition dialog “Cancel” crashes the app | `.copi/specs/fix/prevent-ui-panics/` | `fix/prevent-ui-panics` | `.copi/audits/2026-01-24T00-37-04Z.md` | Spec created |
| GAP-002 | Create-partition dialog can panic on unexpected state | `.copi/specs/fix/prevent-ui-panics/` | `fix/prevent-ui-panics` | `.copi/audits/2026-01-24T00-37-04Z.md` | Spec created |
| GAP-003 | Several menu actions are wired to `todo!()` (crash-on-click) | `.copi/specs/fix/prevent-ui-panics/` | `fix/prevent-ui-panics` | `.copi/audits/2026-01-24T00-37-04Z.md` | Spec created |
| GAP-008 | Device change detection is polling-based (1s) instead of signal-based | `.copi/specs/fix/device-change-detection-signals/` | `fix/device-change-detection-signals` | `.copi/audits/2026-01-24T00-37-04Z.md` | Implemented |
| N/A | Remove old/backup polling for device events logic | `.copi/specs/fix/remove-device-polling-fallback/` | `fix/remove-device-polling-fallback` | N/A (brief) | Implemented |
