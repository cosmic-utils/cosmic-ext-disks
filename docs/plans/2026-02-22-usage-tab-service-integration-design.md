# Usage Tab Service Integration Design

Date: 2026-02-22  
Status: Approved

## Problem Statement

Expose the new global usage scan through the service and wire it into the UI as a new `Usage` tab next to `Volume`.

The Usage tab should:
- show a usage bar chart using generalized segment DTOs
- keep legend optional and disabled for now
- show a color-coded category tab control under the chart
- show a grid of top files (`path`, `size`) for selected category
- show loading progress bar while loading

Scope for now is global scan rooted at `/`, regardless of selection. Filtering comes later.

## Core Constraint

- Canonical payload DTOs live in `storage-common`.
- `storage-sys` must build those DTOs directly (no mirror DTOs in sys).
- UI may define additional local state structs for view state only.

## Goals

- End-to-end service → client → UI flow for global usage data.
- Reuse segment structure between pie and bar consumers via generalized segment naming.
- Keep UI responsive with loading/progress state.
- Preserve clean layering and extensibility for later filtering.

## Non-Goals

- Per-volume/per-mount filtering.
- Legend-enabled UI by default.
- Additional columns beyond path + size in file grid.

## Recommended Approach

Use a service-backed global scan API with progress updates and final result payload, consumed by a new Usage tab in UI.

## Architecture

### 1) Shared DTOs (single source of truth)

In `storage-common`, define canonical models for usage tab payloads:

- `UsageCategory` (enum of category types)
- `UsageSegment` (generalized from pie segment input)
- `UsageTopFileEntry` (`path`, `size_bytes`)
- `UsageCategoryFiles` (`category`, `files`)
- `UsageScanProgress` (`percent`, `bytes_processed`)
- `UsageScanResult` (`segments`, `top_files_by_category`, scan metadata)

`storage-sys` scanner should output these DTOs directly.

### 2) Service / D-Bus API

Add usage-scan methods/signals in service:

- Start or request global usage scan
- Progress updates while scan runs
- Final result retrieval/response

Global scope fixed to `/` in this phase.

### 3) UI client integration

- Add client methods to invoke scan and subscribe/read progress.
- Decode canonical DTOs from service and map into UI state.

### 4) UI composition

Usage tab content (top to bottom):

1. Progress bar (visible while loading)
2. Bar chart (using generalized `UsageSegment` DTO)
3. Optional legend component under chart (`legend_enabled = false` for now)
4. Color-coded category tab control
5. File grid (`path`, `size`), showing selected category files sorted by size desc

## Data Flow

1. User opens `Usage` tab.
2. UI dispatches global usage scan request.
3. UI receives progress snapshots and updates loading bar.
4. UI receives final result and populates chart + tabs + file grid.
5. Selecting a category tab updates file grid view only (no new scan).

## Error Handling

- Service/API error: show inline error in Usage tab with retry control.
- Progress stream interruption before completion: transition to error state with retry.
- Partial category payloads are invalid for final render; require complete payload or error.

## Testing Strategy

1. `storage-sys`: scanner returns canonical `storage-common` DTOs directly.
2. Service/dbus: DTO serialization/deserialization round-trip tests.
3. UI state tests: loading → progress → loaded transitions.
4. UI view tests:
   - Usage tab visible next to Volume tab
   - legend hidden by default
   - category tabs rendered with colors
   - file grid shows path + size sorted by selected category.

## Rollout Notes

- First wire global mode only.
- Keep request/response shape extensible for future filter args.
- Avoid introducing duplicate DTO layers in `storage-sys`.
