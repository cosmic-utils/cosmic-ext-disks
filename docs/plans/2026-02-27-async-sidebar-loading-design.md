# Async Sidebar Loading Design

## Context
The app currently builds all `UiDrive` models serially before publishing a single `UpdateNav` message. On systems with many loop devices this delays first render of drives and makes related sections (including Network) appear unavailable until startup work finishes.

## Goals
- Start startup fetches concurrently instead of chaining them.
- Load drives asynchronously and add them to the sidebar as each drive finishes.
- Preserve deterministic section ordering while allowing item pop-in.
- Preserve current selection behavior (do not reset active/child selection unexpectedly).
- Show per-section loading indicators in sidebar headers so users understand data is still loading.
- Emit `info`-level per-drive build timing logs for performance follow-up.

## Non-Goals
- No redesign of sidebar structure or interaction model.
- No new filters, preferences, or settings for drive classes.
- No backend API changes.

## Selected Approach
Approach A: stream per-drive results and track section loading flags.

### Why this approach
- Delivers the required UX quickly with minimal architecture churn.
- Keeps existing `UiDrive::new` and nav/sidebar models reusable.
- Avoids a large partial-model refactor while still improving perceived performance.

## Architecture

### Startup task orchestration
- Replace init-time `.chain(...)` startup sequencing with concurrent startup tasks (`Task::batch`).
- Split drive loading into start/progress/finish events:
  - drive loading started,
  - one message per completed drive build,
  - drive loading finished.
- Keep network and logical startup loads independent from drive completion.

### Incremental drive loading pipeline
- Add a loader path that:
  - lists disks once,
  - builds each disk with `UiDrive::new` in independent async tasks,
  - emits completion message per disk with success/error + elapsed time,
  - emits completion message after all tasks resolve.
- Continue-on-error behavior remains: failed drives log and do not block others.

### Sidebar loading state model
- Extend sidebar state with section loading flags:
  - logical loading,
  - drive sections loading,
  - network loading,
  - images loading (derived from drives loading).
- Keep section classification unchanged (`Internal`, `External`, `Images`, `Logical`, `Network`).

### UI behavior
- Sidebar section headers render an animated spinner to the right while that section is loading.
- Drives pop into sections as they finish building.
- Sorting remains deterministic per section while items arrive.
- Existing row controls and expansion behavior remain unchanged.

## Data Flow
1. App init dispatches startup tasks in parallel.
2. Drive loader emits `DriveLoadStarted(total)`.
3. Each drive task emits `DriveLoaded { result, elapsed_ms }`.
4. Update layer merges successful drives into sidebar/nav incrementally and preserves active selection.
5. Drive loader emits `DriveLoadFinished` and clears section loading state.
6. Network/logical completion messages independently clear their own header spinners.

## Logging
- Add `info` logs per drive build with:
  - device path,
  - section classification,
  - elapsed milliseconds,
  - success/failure.
- Keep existing warning/error logs for failed calls.

## Error Handling
- Any single drive failure logs warning and does not fail global startup.
- If disk listing fails, end drive loading with empty set and set drive loading flags false.
- Network/logical failures preserve current behavior but still clear loading indicators.

## Testing Strategy
- Unit tests for deterministic incremental ordering logic in sidebar state updates.
- Unit tests for section loading flag transitions:
  - started -> in-progress -> finished,
  - independent transitions for logical/network.
- Manual verification:
  - launch app with loop-heavy environment,
  - confirm drives appear progressively,
  - confirm spinner visibility and disappearance per section,
  - confirm `info` timing logs appear per drive.

## Risks and Mitigations
- **Risk:** nav rebuild churn when each drive arrives.
  - **Mitigation:** update only with incremental state + deterministic ordering; keep code paths focused.
- **Risk:** spinner implementation mismatch with theme primitives.
  - **Mitigation:** use existing COSMIC loading/spinner primitive only; no custom animation styles.
- **Risk:** race in selection preservation.
  - **Mitigation:** preserve current selected drive/child keys on each incremental update.

## Success Criteria
- First drives appear before full drive scan completes.
- Network/logical loads are no longer blocked by drive loading.
- Sidebar shows section-level loading indicators during fetch.
- `info` logs include per-drive build timings.
- Selection and existing sidebar controls continue to behave as before.
