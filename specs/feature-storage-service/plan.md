# Implementation Plan: Refactor Build Workflow & UI Analysis

**Branch**: `feature/storage-service` | **Date**: 2026-02-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/feature-storage-service/spec.md`

## Summary

Refactor the justfile to provide a streamlined default development workflow and analyze the storage-ui crate for architectural improvements. The default `just` command will execute the complete development cycle (build → install policies → stop service → start service bg → launch UI). Additionally, identify redundancy in the justfile and document overcomplexity patterns in storage-ui for future refactoring.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2024)
**Primary Dependencies**: cargo, just (task runner), systemd, dbus, polkit
**Storage**: N/A (no data storage changes)
**Testing**: `cargo test --workspace`, manual workflow testing
**Target Platform**: Linux (systemd-based distributions)
**Project Type**: Multi-crate workspace with justfile build automation
**Performance Goals**: N/A (developer experience improvement)
**Constraints**: Must maintain backward compatibility with existing justfile recipes
**Scale/Scope**: 1 justfile (~230 lines), 1 UI crate (~70 source files)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Data Safety First | ✅ PASS | No disk operations modified; only build workflow changes |
| II. Modular Crate Architecture | ✅ PASS | No crate boundary changes |
| III. Quality Gates | ✅ PASS | Must verify `cargo test`, `cargo clippy`, `cargo fmt` pass after changes |
| IV. Evidence-Based Design | ✅ PASS | Analysis will cite specific files and line numbers |
| V. Linux System Integration | ✅ PASS | Uses existing D-Bus/Polkit installation paths |

**Gate Status**: ✅ All gates pass. Proceed to implementation.

## Project Structure

### Documentation (this feature)

```text
specs/feature-storage-service/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # UI analysis findings
└── tasks.md             # Implementation tasks
```

### Source Code (repository root)

```text
justfile                        # Task runner definitions (PRIMARY TARGET)
storage-ui/
├── src/
│   ├── app.rs                  # App re-exports
│   ├── main.rs                 # Entry point
│   ├── client/                 # D-Bus client modules
│   │   ├── mod.rs
│   │   ├── btrfs.rs
│   │   ├── disks.rs
│   │   ├── error.rs
│   │   ├── filesystems.rs
│   │   ├── image.rs
│   │   ├── luks.rs
│   │   ├── lvm.rs
│   │   └── partitions.rs
│   ├── models/                 # UI data models
│   │   ├── mod.rs
│   │   ├── helpers.rs
│   │   ├── load.rs
│   │   ├── ui_drive.rs
│   │   └── ui_volume.rs
│   ├── ui/                     # UI components
│   │   ├── mod.rs
│   │   ├── app/                # Main app state/update/view
│   │   │   ├── mod.rs
│   │   │   ├── message.rs
│   │   │   ├── state.rs
│   │   │   ├── subscriptions.rs
│   │   │   ├── view.rs
│   │   │   └── update/         # Update handlers (7 files)
│   │   ├── btrfs/              # BTRFS management UI
│   │   ├── dialogs/            # Dialog components
│   │   ├── sidebar/            # Custom sidebar treeview
│   │   └── volumes/            # Volume display/control
│   │       ├── mod.rs
│   │       ├── message.rs
│   │       ├── state.rs
│   │       ├── view.rs
│   │       └── update/         # Update handlers (8 files)
│   ├── utils/                  # Utility functions
│   └── views/                  # Additional views
└── build.rs
```

**Structure Decision**: Existing workspace structure preserved. Changes limited to `justfile` and documentation. Analysis covers `storage-ui/src/` as specified.

## Complexity Tracking

No violations to justify. This refactoring reduces complexity.

## Implementation Approach

### Phase 1: Justfile Refactor (US1 + US2)

1. **Create new default recipe** that chains existing recipes:
   - `build` → `install-dev-policies` → `stop-service` → `start-service-bg` → `start-app`

2. **Remove redundancy** by making recipes depend on each other rather than duplicating logic:
   - `dev` should call `build` + `start-service-bg` + `start-app` instead of inlining
   - Remove duplicate `cargo build --workspace` calls

3. **Preserve existing workflows**:
   - `just dev` - full dev cycle (build, service, app)
   - `just start-service-bg` - background service only
   - `just start-app` - UI only (assumes service running)

### Phase 2: Storage-UI Analysis (US3)

Analysis areas:

1. **Message Routing Complexity**
   - File: `storage-ui/src/ui/volumes/message.rs` (188 lines, 15+ From impls)
   - File: `storage-ui/src/ui/app/message.rs` (181 lines, 60+ variants)
   - Pattern: Messages wrap through multiple layers (DialogMessage → VolumesControlMessage → Message)

2. **State Management**
   - File: `storage-ui/src/ui/volumes/state.rs` (434 lines, VolumesControl struct)
   - File: `storage-ui/src/ui/app/state.rs` (57 lines, AppModel struct)
   - Pattern: Nested state with BtrfsState, SidebarState as subcomponents

3. **Update Handler Organization**
   - Directory: `storage-ui/src/ui/volumes/update/` (8 files, ~77KB total)
   - Directory: `storage-ui/src/ui/app/update/` (5+ files, ~75KB total)
   - Pattern: Split by domain (btrfs, encryption, filesystem, mount, partition)

4. **Convention Adherence**
   - COSMIC pattern: Application trait, Core state, Task<Message>
   - Current state: Follows pattern but with deep nesting

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking existing dev workflows | Medium | High | Test all justfile recipes after changes |
| Incomplete UI analysis | Low | Low | Follow spec requirements (FR-006, FR-007, FR-008) |
| Sudo failures in default recipe | Medium | Medium | Add clear error messages for policy installation failures |
