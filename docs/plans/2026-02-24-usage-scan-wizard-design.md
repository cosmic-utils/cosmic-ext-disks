# Usage Scan Single-Page Wizard Design

## Summary
Add a single-page wizard shown before every Usage scan in-app. The wizard allows users to choose mount points to index, toggle `Show All Files`, and select scan parallelism (`Low`, `Balanced`, `High`).

The layout should follow the same visual style cues as the existing rclone wizard (headline, form body, footer actions) while remaining inside the Usage tab flow.

## Goals
- Show a pre-scan wizard on every scan start (`Refresh`).
- Let users choose one or more mount points per scan run.
- Include `Show All Files` and parallelism controls in the wizard.
- Preserve existing scan progress/result behavior after scan starts.

## Non-Goals
- No multi-step wizard; this is a single page.
- No new top-level app route.
- No background auto-scan when wizard fields change.
- No additional filters beyond mount points, show-all, and parallelism.

## Architecture

### UI state and view
Extend existing Usage tab state with a wizard substate:
- `wizard_open: bool`
- `wizard_mount_points: Vec<(mount: String, selected: bool)>`
- `wizard_show_all_files: bool`
- `wizard_parallelism: UsageScanParallelismPreset`
- `wizard_error: Option<String>`

`UsageRefreshRequested` opens the wizard instead of launching a scan directly.

### Wizard component placement
Render wizard content in the Usage view when `wizard_open == true`, replacing the action-bar/file-list area for that moment. Keep this in the same Usage tab (no modal required).

### Scan request payload
Usage scan initialization includes:
- selected mount points
- show-all flag
- parallelism preset

Service uses selected mount points as roots, plus existing auth and scan-progress behavior.

## Data Flow
1. User clicks `Refresh` in Usage tab.
2. App opens wizard and loads mount-point options.
3. Wizard defaults:
   - all mount points selected,
   - show-all from saved setting,
   - parallelism from saved setting.
4. User edits fields.
5. `Start Scan` validates at least one mount selected.
6. On valid input, wizard closes and scan starts with selected settings.
7. Existing progress/result rendering continues unchanged.
8. `Cancel` closes wizard and leaves previous scan result intact.

## UX Details
- Form fields:
  - Mount points (multi-select list of checkboxes)
  - `Show All Files` (checkbox)
  - `Parallelism` (dropdown: Low/Balanced/High)
- Footer actions:
  - `Cancel`
  - `Start Scan` (disabled until at least one mount selected)
- Inline validation and errors remain inside the wizard body.

## Error Handling
- Mount discovery failure: show wizard inline error and disable `Start Scan`.
- Empty mount selection: show validation hint and keep `Start Scan` disabled.
- Show-all auth denial on start: keep wizard open and show inline error.
- Scan launch failure: keep wizard state visible with actionable error.

## Testing Strategy

### UI state/reducer tests
- `UsageRefreshRequested` opens wizard instead of scanning immediately.
- `Start Scan` with valid selection dispatches scan load with expected payload.
- `Cancel` keeps prior scan result untouched.
- Zero mount selection blocks start.

### Service tests
- Scan roots are limited to selected mount points.
- Parallelism mapping still applies for wizard-originated requests.

### Manual smoke checks
- Refresh always opens wizard.
- Selecting subset of mounts changes resulting indexed scope.
- Show-all and parallelism choices affect scan behavior as expected.
