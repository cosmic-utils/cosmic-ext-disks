# Implementation Plan: RClone Mount Management

**Branch**: `072-rclone-mounts` | **Date**: 2026-02-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/072-rclone-mounts/spec.md`

## Summary

Add RClone mount management capability to the COSMIC Ext Storage application. This includes reading/writing rclone.conf files, controlling RClone mounts (start/stop/restart), testing remote configurations, and displaying mounts in a "Network" section of the sidebar. The feature supports both per-user and system-wide configurations with polkit elevation for system operations.

## Technical Context

**Language/Version**: Rust (stable channel, edition 2024)
**Primary Dependencies**: zbus, zbus-polkit, libcosmic, tokio, serde, serde_json, toml
**Storage**: File-based (rclone.conf INI/TOML format)
**Testing**: cargo test, cargo clippy
**Target Platform**: Linux (systemd-based distributions)
**Project Type**: Workspace with multiple crates (service + UI)
**Performance Goals**: <2s to display remotes, <5s mount operations, <10s config tests
**Constraints**: Must not spawn own RClone daemon; interact with existing systemd services
**Scale/Scope**: Per-user and system-wide RClone configurations

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Data Safety First | PASS | Mount operations are non-destructive; config writes validate before saving |
| II. Modular Crate Architecture | PASS | RClone code split appropriately: storage-sys (low-level), storage-service (D-Bus), storage-common (types), storage-ui (UI) |
| III. Quality Gates | PASS | Will follow existing test/clippy/fmt requirements |
| IV. Evidence-Based Design | PASS | This plan cites specific files and patterns from existing codebase |
| V. Linux System Integration | PASS | Uses D-Bus/Polkit patterns consistent with existing storage-service |

## Project Structure

### Documentation (this feature)

```text
specs/072-rclone-mounts/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # D-Bus interface contracts
│   └── rclone-api.md
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
storage-common/
├── src/
│   ├── rclone.rs        # NEW: RClone data models (RemoteConfig, NetworkMount, MountStatus)
│   └── lib.rs           # MODIFIED: Add pub mod rclone

storage-sys/
├── src/
│   ├── rclone.rs        # NEW: Low-level rclone CLI operations
│   ├── lib.rs           # MODIFIED: Add pub mod rclone
│   └── error.rs         # MODIFIED: Add RCloneError variants

storage-service/
├── src/
│   ├── rclone.rs        # NEW: D-Bus interface for RClone operations
│   ├── main.rs          # MODIFIED: Register RCloneHandler at /org/cosmic/ext/StorageService/rclone
│   └── service.rs       # MODIFIED: Add "rclone" to supported_features

data/
├── polkit-1/actions/
│   └── org.cosmic.ext.storage-service.policy  # MODIFIED: Add 4 RClone polkit actions

storage-ui/
├── src/
│   ├── sidebar/
│   │   └── network_section.rs  # NEW: Network section in sidebar
│   └── pages/
│       └── network_mounts.rs   # NEW: RClone mount management page
```

**Structure Decision**: Following existing patterns from btrfs/filesystems handlers. RClone types in storage-common, CLI operations in storage-sys, D-Bus interface in storage-service, UI components in storage-ui.

## Complexity Tracking

No constitution violations requiring justification.
