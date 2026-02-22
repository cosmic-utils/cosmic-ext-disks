# Usage Action Bar Design

## Summary
Add an action/settings bar above Usage category tabs with controls for `Show All Files`, `Delete`, `Clear Selection`, `Number of files`, and `Refresh`.

This design extends existing Usage tab flow using the current state/message architecture (Approach A), minimizing churn while adding privileged listing/deletion behavior and richer file-list interaction.

## Goals
- Keep default behavior limited to user-accessible files.
- Support admin-gated all-file listing.
- Support standard list selection behavior: single, Shift-range, Ctrl-toggle.
- Enable deletion of selected files with mixed-ownership handling.
- Keep scan parameters explicit (`show_all_files`, `top_files_per_category`) and applied on Refresh.

## Non-Goals
- No new pages/dialog families beyond existing message/dialog patterns.
- No background auto-refresh from control edits.
- No extra filtering/sorting dimensions beyond existing category tabs.

## Architecture
Use existing `UsageTabState` and app message/update pipeline.

### State additions (UI)
Add usage-control and selection fields to `UsageTabState`:
- `show_all_files: bool`
- `show_all_files_authorized_for_session: bool`
- `top_files_per_category: u32`
- `selected_paths: BTreeSet<String>` (or `HashSet<String>`)
- `selection_anchor_index: Option<usize>`
- `deleting: bool`
- `last_operation_summary: Option<String>`

### Message additions (UI)
Add messages for:
- control changes (`UsageSetShowAllFiles`, `UsageSetTopFilesPerCategory`, `UsageRefresh`)
- selection updates (`UsageSelectRow`, `UsageToggleRow`, `UsageRangeSelect`, `UsageClearSelection`)
- delete flow (`UsageDeleteSelected`, `UsageDeleteCompleted`)
- auth outcomes for all-files toggle.

### Service/API changes
Extend usage-scan call to include:
- `show_all_files`
- `top_files_per_category`

Add delete API for selected file paths with per-path outcomes.

## Data Flow
1. User edits controls in action bar.
2. `Number of files` and `Show All Files` update local state; `Show All Files` toggle-on requests admin if not already session-authorized.
3. `Refresh` dispatches usage scan using current control values.
4. Results update category totals and file lists; selection persists only for paths still visible (or is pruned).
5. `Delete` sends selected paths to service; service applies mixed-ownership rule with auth and returns per-file results.
6. UI shows summary and updates list on success (`Refresh` or local prune).

## UX Specification
Place action bar directly above category tabs in Usage tab:
- Toggle: `Show All Files`
- Button: `Delete`
- Button: `Clear Selection`
- Numeric input: `Number of files`
- Button: `Refresh`

Rules:
- Default: `Show All Files` off.
- Toggle-on prompts admin once per session.
- `Delete` enabled only when selection non-empty.
- `Refresh` is explicit trigger for scan with current controls.

## Selection Behavior
- Click: select one row and set anchor.
- Ctrl-click: toggle row selection.
- Shift-click: select contiguous range from anchor.
- Clear Selection: clear selected set + anchor.

Selection key is full file path (stable identity).

## Auth and Safety
- Listing all files requires admin when toggling `Show All Files` on.
- Session authorization is cached for repeated refreshes.
- Mixed delete:
  - if any selected file requires privilege, request admin and proceed for all selected on success,
  - on denied/cancelled auth, abort delete and preserve selection.

## Error Handling
- Scan errors: preserve prior successful data, show non-blocking error.
- Delete errors: display per-file outcome summary (deleted/failed/not-found/permission denied).
- Concurrent actions: disable duplicate refresh/delete while operation is in progress.

## Testing Strategy
### UI state tests
- Show-all toggle auth success/failure rollback.
- Number-of-files persistence until Refresh.
- Single/Ctrl/Shift selection behavior.
- Clear Selection reset.

### Client/service tests
- Scan args encode/decode with new parameters.
- Delete API parse/serialize with per-file results.

### Service tests
- Ownership-based privilege branching.
- Mixed selection delete with and without auth.

### Manual smoke checks
- Action bar rendering and enabled/disabled states.
- Refresh applies file-count and show-all controls.
- Delete + selection UX correctness.
