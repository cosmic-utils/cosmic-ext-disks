# Usage Scan Parallelism Preset Design

## Summary
Add a persisted Settings option for usage scan parallelism with three presets: `Low`, `Balanced`, and `High`.

The selected preset is passed with usage scan requests and mapped in service at scan start to a concrete thread count derived from current CPU availability.

## Goals
- Provide a simple user-facing performance control in Settings.
- Persist the choice across app restarts using existing config patterns.
- Keep scanner behavior explicit and deterministic per scan request.
- Avoid introducing new pages, dialogs, or background systems.

## Non-Goals
- No per-scan custom numeric thread input.
- No live benchmarking or adaptive runtime tuning.
- No changes to usage scan result schema.

## Architecture

### Shared API model
Add a shared enum for preset selection used across UI/client/service:
- `UsageScanParallelismPreset::Low`
- `UsageScanParallelismPreset::Balanced`
- `UsageScanParallelismPreset::High`

Usage scan request includes this preset.

### UI settings/config
Extend app config with persisted field:
- `usage_scan_parallelism: UsageScanParallelismPreset` (default `Balanced`)

Settings pane adds a control under existing Settings:
- Label: `Scan Parallelism`
- Choices: `Low`, `Balanced`, `High`

Changing the setting updates in-memory config and persists with the same `write_entry` pattern used by existing toggles.

### Usage scan call path
When dispatching `UsageScanLoad`, app reads current preset from config and passes it to client/service request.

### Service mapping to threads
Service computes concrete `threads` at scan start using CPU count `n`:
- `low = max(1, ceil(n/4))`
- `balanced = max(1, ceil(n/2))`
- `high = max(1, n)`

The computed value is assigned to existing `ScanConfig.threads` and scan executes normally.

## Data Flow
1. App starts and loads persisted config (default `Balanced` if absent).
2. Settings view renders current preset.
3. User changes preset in Settings.
4. App updates config state and persists it.
5. User triggers usage refresh (or existing auto-refresh trigger).
6. App sends scan request with current preset.
7. Service maps preset to threads based on live CPU count and runs scanner.
8. Existing scan progress/result handling remains unchanged.

## Error Handling
- Invalid/missing persisted config value falls back to default `Balanced`.
- If preset decoding fails in request parsing, service rejects request with explicit error.
- CPU count edge cases clamp to at least one thread.

## Testing Strategy

### Unit tests
- Enum serde roundtrip for preset in shared model.
- Service mapping tests for representative CPU counts:
  - 1 core
  - 2 cores
  - 8 cores
  - odd count (e.g. 6 or 10) to validate ceil behavior

### UI/config tests
- Settings message updates config preset.
- Config write/read roundtrip preserves preset.

### Integration checks
- Client/service usage scan signature compiles with new preset argument.
- End-to-end usage scan still compiles and returns unchanged result schema.

## Implementation Notes
- Keep mapping logic in service to centralize policy for all callers.
- Keep scanner unaware of presets; scanner receives only concrete thread count.
- Keep UX minimal: one Settings control, no additional hints/tooltips unless requested.
