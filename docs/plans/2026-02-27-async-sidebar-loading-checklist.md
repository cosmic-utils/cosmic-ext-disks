# Async Sidebar Loading Checklist

**Date:** 2026-02-27  
**Scope:** Action-level checklist for parallel startup loading, incremental drive pop-in, section loading spinners, and per-drive timing logs.

---

## A. Preparation

- [x] Confirm approved design doc exists: `docs/plans/2026-02-27-async-sidebar-loading-design.md`.
- [x] Confirm approved implementation plan exists: `docs/plans/2026-02-27-async-sidebar-loading-implementation-plan.md`.
- [x] Confirm branch scope is limited to async startup/sidebar perf UX + logging.
- [x] Run baseline compile and record result timestamp: `cargo check -p storage-app`.
- [x] Run baseline app startup capture and record qualitative behavior in section L:
  - [x] `RUST_LOG=cosmic_ext_storage=info,storage_contracts=info just app`
  - [x] Record observed time-to-first-drive and spinner/empty-state behavior.

## B. Global Guardrails

- [x] Preserve existing sidebar section taxonomy and order:
  - Logical
  - Internal
  - External
  - Network
  - Images
- [x] Preserve existing selection semantics (active drive + child selection) during incremental updates.
- [x] Keep deterministic in-section ordering while drives pop in.
- [x] Do not introduce new settings, filters, or unrelated UI changes.
- [x] Use existing COSMIC widgets/theme primitives only (no custom colors/shadows/fonts).
- [x] Keep `load_all_drives()` behavior intact for existing full-refresh callers unless explicit migration is required.

## C. Message Contract Updates

## C1. Add startup streaming messages

- [x] Modify `storage-app/src/message/app.rs` with incremental drive startup variants:
  - [x] `DriveLoadStarted { total: usize }`
  - [x] `DriveLoaded { result: Result<UiDrive, String>, elapsed_ms: u128 }`
  - [x] `DriveLoadFinished`
- [x] Ensure naming is consistent with existing message conventions.
- [x] Ensure message variants are wired through any exhaustive `match` handling.

## C2. Maintain compatibility with existing refresh messages

- [x] Keep existing `UpdateNav` and `UpdateNavWithChildSelection` message paths functional.
- [x] Ensure incremental path does not break post-operation full refresh flows.

## D. Sidebar State Loading Flags

## D1. Add section loading state

- [x] Modify `storage-app/src/state/sidebar.rs` to track section-level loading booleans.
- [x] Ensure all section types can represent loading state independently.
- [x] Add helper methods for startup transitions:
  - [x] mark startup-loading begin
  - [x] mark per-section completion
  - [x] mark startup-loading complete

## D2. Unit tests for state transitions

- [x] Add/extend tests in `storage-app/src/state/sidebar.rs` covering:
  - [x] default flags
  - [x] drive-loading start/finish transitions
  - [x] independent logical/network loading transitions
  - [x] no stale loading flags after completion

## E. Incremental Drive Loader

## E1. Introduce incremental drive build path

- [x] Modify `storage-app/src/models/load.rs` to support per-drive completion events for startup.
- [x] Keep disk enumeration as a single list operation.
- [x] Build each drive in independent async tasks.
- [x] Continue-on-error for individual drive failures.

## E2. Add per-drive timing logs

- [x] Emit `info` log per drive build with at least:
  - [x] device path
  - [x] elapsed milliseconds
  - [x] success/failure
  - [x] section classification (if available at logging point)
- [x] Keep warning/error logs for failure diagnostics.

## E3. Deterministic ordering helper

- [x] Add helper logic/tests to preserve stable sort order as drives arrive.
- [x] Verify sort behavior does not oscillate as additional drives complete.

## F. Startup Task Orchestration

## F1. Parallelize startup tasks

- [x] Modify `storage-app/src/app.rs` init path to dispatch startup work in parallel (`Task::batch` or equivalent).
- [x] Ensure drive loading does not block network/logical startup tasks.
- [x] Keep filesystem tools startup behavior intact.

## F2. Drive startup lifecycle events

- [x] Dispatch `DriveLoadStarted` before incremental drive tasks begin.
- [x] Dispatch `DriveLoaded` per completed drive task.
- [x] Dispatch `DriveLoadFinished` when all drive tasks have resolved.
- [x] Ensure failure in one drive task does not suppress final completion message.

## G. Update Flow Integration

## G1. Handle new incremental drive messages

- [x] Modify `storage-app/src/update/mod.rs` to handle:
  - [x] `DriveLoadStarted`
  - [x] `DriveLoaded`
  - [x] `DriveLoadFinished`
- [x] On `DriveLoaded(Ok)`, merge drive and refresh nav/sidebar incrementally.
- [x] On `DriveLoaded(Err)`, log and continue without user-visible hard failure.
- [x] On `DriveLoadFinished`, clear drive/images loading flags.

## G2. Preserve selection on incremental nav rebuilds

- [x] Modify `storage-app/src/update/nav.rs` (or helper path) to preserve active drive selection across each incremental update.
- [x] Preserve child selection where possible and avoid spurious resets.
- [x] Ensure behavior remains compatible with dialog-running state guards.

## G3. Network/logical loading flag completion

- [x] Ensure logical success/failure handlers clear logical loading spinner state.
- [x] Ensure network success/failure handlers clear network loading spinner state.
- [x] Ensure section states cannot remain indefinitely loading after error.

## H. Sidebar Header Spinner UI

## H1. Header rendering updates

- [x] Modify `storage-app/src/views/sidebar.rs` section header rendering to include an animated spinner right of the header label when section is loading.
- [x] Apply to all section types represented in sidebar state.
- [x] Keep existing section-specific actions (e.g., image/network add buttons) functional.

## H2. Network section consistency

- [x] Modify `storage-app/src/views/network.rs` as needed so network header spinner behavior aligns with other sections.
- [x] Ensure loading text/empty/error messaging still follows existing UX rules.

## I. Regression and Compatibility Checks

- [x] Verify no compile regressions in other update paths that still use full `UpdateNav` refreshes.
- [x] Verify no behavioral regressions for:
  - [x] drive add/remove events
  - [x] volume mount/unmount refreshes
  - [x] logical operation refresh cycles
- [x] Verify no new clippy/fmt violations introduced by changes.

## J. Verification Commands

- [x] `cargo check -p storage-app`
- [x] `cargo test -p storage-app`
- [x] `cargo fmt --all --check`
- [x] Optional: `cargo clippy -p storage-app --all-targets`

## K. Runtime Validation (Manual)

- [x] Start service if required: `just service`.
- [x] Run app with info logs: `RUST_LOG=cosmic_ext_storage=info,storage_contracts=info just app`.
- [ ] Validate expected runtime behavior:
  - [ ] section headers show spinner while loading
  - [x] drives appear incrementally (pop-in)
  - [x] deterministic order maintained while loading
  - [ ] selection remains stable during incremental updates
  - [x] network/logical availability no longer blocked by drive loading
- [x] Confirm per-drive timing logs are emitted at info level.

## L. Verification Evidence (fill during execution)

- [x] Baseline compile timestamp/result: `cargo check -p cosmic-ext-storage` PASS.
- [x] Baseline startup behavior notes: drive-heavy startup previously blocked visible updates until full completion.
- [x] Post-change compile timestamp/result: `cargo check -p cosmic-ext-storage` PASS; `cargo check` PASS.
- [x] Post-change test timestamp/result: `cargo test -p cosmic-ext-storage` PASS (39 passed).
- [x] Post-change runtime behavior notes: incremental drive completions observed via interleaved per-drive timing + nav updates; network/logical load no longer serialized behind full drive completion.
- [x] Example per-drive timing log lines captured: `drive build complete device=/dev/loop7 section="images" elapsed_ms=2632`, `drive build complete device=/dev/nvme0n1 section="internal" elapsed_ms=2686`.
- [x] Formatting gate note: `cargo fmt --all --check` now passes after warning/error cleanup pass.

## M. Done Definition

- [x] Startup tasks execute concurrently rather than strictly chained.
- [x] Sidebar receives drive entries incrementally as each drive build completes.
- [x] All sidebar section types support visible loading indication.
- [ ] Section headers show animated spinner at right while loading.
- [x] Per-drive build timing is logged at info level.
- [x] Existing selection behavior and operational refresh flows remain intact.
- [x] `storage-app` checks/tests pass with no unrelated scope creep.
