# Usage UI Tweaks Design

## Summary
Refine the Usage tab UX in three areas: loading screen, pre-scan wizard, and result screen layout/actions.

This design explicitly treats the existing rclone wizard implementation as the canonical source for wizard layout and styling. Usage wizard styling should be rebuilt from shared wizard generics derived from rclone patterns, not from the current usage wizard appearance.

## Goals
- Center the loading UI and improve progress information readability.
- Align Usage wizard spacing/sizing/layout with rclone wizard visuals.
- Replace usage mount-point checkboxes with wrapping toggle tiles.
- Introduce shared wizard generics used by both rclone and usage to reduce duplication and enforce visual coherence.
- Improve Usage result-screen layout for category tabs, action bar ordering/alignment, and file list sizing/typography.

## Non-Goals
- No new pages, dialogs, or feature flags.
- No changes to usage scan semantics or service contracts beyond current wizard options.
- No redesign of the rclone wizard flow steps.
- No new themes, colors, or typography systems beyond existing COSMIC primitives.

## Canonical Styling Baseline
- **Source of truth:** `storage-ui/src/ui/network/view.rs` wizard layout and controls.
- Shared wizard generics are extracted from this baseline.
- Usage wizard must adopt these shared components and measurements.
- Existing usage wizard styling is ignored as a baseline.

## Architecture

### Shared Wizard Generics (Deep, Focused)
Introduce a shared wizard UI module in `storage-ui` that provides:

- `WizardShell`
  - Title/header area
  - Optional progress element
  - Scrollable content region
  - Footer action row
  - Fill-height layout and consistent outer padding
- `WizardActionRow`
  - Canonical alignment and spacing for cancel/back/next/start/create actions
  - Left/right grouping behavior reused across wizards
- `OptionTileGrid`
  - Wrapping tile layout (responsive)
  - Canonical tile spacing and card geometry based on rclone type cards
- `SelectableTile` styling primitive
  - Shared selected/unselected visual states
  - Used for rclone provider cards and usage mount-point toggles

### Ownership Boundaries
- Keep feature state/messages local:
  - usage continues with usage-specific messages/state
  - network/rclone continues with `NetworkMessage` and wizard step state
- Shared generic components are message-type generic and stateless UI builders.
- Behavior/state machines remain in existing reducers to avoid coupling business logic.

## UX Specification

### 1) Loading Screen
- Usage loading content is centered both vertically and horizontally in the available Usage tab body.
- Progress bar width is constrained to a sensible maximum width (responsive), not full-width on large windows.
- First row is a two-sided row:
  - left: `Scanning disk usage...`
  - right: current/total byte text
- Progress bar appears directly below this row.

### 2) Usage Wizard
- Recompose usage wizard with shared `WizardShell` derived from rclone wizard layout.
- Replace mount-point checkboxes with wrapping toggle tiles via shared `OptionTileGrid` + `SelectableTile`.
- Toggle tile behavior:
  - multi-select on click
  - selected state visually obvious, using shared button class primitives
- Keep existing fields:
  - Show All Files
  - Parallelism (Low/Balanced/High)
  - Cancel / Start Scan actions
- Start Scan enabled only when mount selection is non-empty and mount loading is complete.

### 3) Usage Screen Layout
- Category tabs are directly below the usage bar.
- Category tabs use true wrapping based on available width (not fixed chunked rows).
- Action bar follows this order/alignment:
  - left group: `Number of files`, `Refresh`
  - right group: `Selected: N`, `Clear Selection`, `Delete`
- `Delete`, `Refresh`, `Clear Selection` buttons use action-button styling consistent with drive header/volume control patterns.
- File-list scroll area fills remaining vertical height in the view.
- Disable full-page scrolling for this section pattern; only file list scrolls.
- File header row typography:
  - normal text size, bold
- File row typography:
  - normal text size

## Data Flow and State Impact
- No new backend data model needed for these tweaks.
- Existing usage wizard state fields remain sufficient for behavior.
- UI state additions may be needed only for layout/adaptation plumbing (e.g., local helper structs for tile rendering), not for business logic.
- Selection, refresh, delete, and wizard-start flows remain on current message/update paths.

## Error Handling
- Preserve current inline wizard error handling behavior.
- Preserve current scan/deletion operation status and errors in Usage view.
- If mount points fail to load, keep Start Scan disabled and show existing inline error style.

## Testing Strategy

### Unit/Reducer Coverage
- Existing usage action and selection tests remain unchanged in intent.
- Add focused tests where practical for any new helper functions (e.g., layout-independent selection/tile mapping helpers).

### UI/Compile Verification
- `cargo check -p cosmic-ext-storage` for UI compile safety.
- Verify shared wizard generic module compiles with both usage and network wizard call sites.

### Manual Smoke Checks
- Loading state is centered and progress text is right-aligned on the title row.
- Usage wizard visually matches rclone wizard spacing/sizing patterns.
- Mount-point tiles wrap with window width and toggle correctly.
- Category tabs wrap naturally under usage bar.
- Action bar order/alignment matches spec.
- File list occupies remaining height and only list scrolls.
- Header/body typography sizing matches spec.

## Risks and Mitigations
- **Risk:** Deep generic extraction may accidentally alter rclone wizard behavior.
  - **Mitigation:** Keep rclone flow/state machine untouched; only move shared presentation primitives.
- **Risk:** Iced layout constraints can make fill-height behavior brittle.
  - **Mitigation:** Use explicit `Length::Fill` structure in Usage tab body and isolate scroll region.
- **Risk:** Wrap behavior differences between fixed rows and flex wrapping may affect small widths.
  - **Mitigation:** Reuse existing `flex_row`/wrap approach used in rclone wizard type grid.

## Acceptance Criteria
- Loading UI is centered and progress text is on the same row as scan label, right-aligned.
- Usage wizard uses shared wizard generics rooted in rclone wizard styling.
- Usage mount points are rendered as wrapping toggle tiles (no checkboxes).
- Category tabs wrap based on width directly below usage bar.
- Action bar order/alignment and button styling match requested pattern.
- File list fills remaining height with contained scrolling.
- File headers and rows use normal text size, with bold header labels.