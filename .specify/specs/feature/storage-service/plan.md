# Implementation Plan: Audit 2026-02-14 Gap Remediation

**Branch**: `feature/storage-service` | **Date**: 2026-02-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `.copi/specs/fix/audit-2026-02-14-gaps/spec.md`

## Summary

Remediate 15 gaps identified in the 2026-02-14 architecture audit (GAP-002 through GAP-016). Focus areas:
- Eliminate runtime panics from unwrap/expect usage
- Fix volume hierarchy display (parent_path population)
- Establish consistent client ownership model
- Add input validation and error context
- Create integration test scaffolding

## Technical Context

**Language/Version**: Rust stable, edition 2024
**Primary Dependencies**: zbus 5.x, udisks2 0.3.x, libcosmic (git), tokio 1.x
**Storage**: N/A (no database)
**Testing**: cargo test, integration tests in tests/
**Target Platform**: Linux (systemd-based)
**Project Type**: Multi-crate workspace
**Performance Goals**: No regressions; async operations non-blocking
**Constraints**: No breaking changes to D-Bus interfaces
**Scale/Scope**: 6 crates, ~15k LOC

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Data Safety First (NON-NEGOTIABLE)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| No unwrap/panic in production paths | ❌ VIOLATED | GAP-002, GAP-003, GAP-005, GAP-011 |
| Destructive ops require confirmation | ✅ PASS | Existing code has confirmation dialogs |
| Error handling comprehensive | ❌ VIOLATED | GAP-005, GAP-012 |
| Partial failures handled | ⚠️ PARTIAL | GAP-016 needs documentation |

**Justification**: This spec directly addresses these violations.

### Principle II: Modular Crate Architecture

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Single purpose per crate | ✅ PASS | Workspace structure clear |
| Unidirectional dependencies | ⚠️ PARTIAL | GAP-009 conversions.rs bridging gap |
| Shared code in storage-models | ✅ PASS | Pattern followed |

### Principle III: Quality Gates (NON-NEGOTIABLE)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| cargo test passes | ⚠️ PARTIAL | GAP-015 missing integration tests |
| cargo clippy passes | ⚠️ PARTIAL | Some warnings from unwrap usage |
| cargo fmt passes | ✅ PASS | Formatting consistent |

### Principle IV: Evidence-Based Design

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Decisions documented with evidence | ✅ PASS | This spec references audit |
| Code comments explain "why" | ⚠️ PARTIAL | GAP-010 orphaned TODOs |

### Principle V: Linux System Integration

| Requirement | Status | Evidence |
|-------------|--------|----------|
| D-Bus/UDisks2 for operations | ✅ PASS | Pattern followed |
| Handle D-Bus failures gracefully | ⚠️ PARTIAL | GAP-012 error context |

## Project Structure

### Documentation (this feature)

```text
.copi/specs/fix/audit-2026-02-14-gaps/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Research findings
└── tasks.md             # Task breakdown
```

### Source Code (repository root)

```text
storage-ui/
├── src/
│   ├── models/
│   │   ├── ui_drive.rs      # GAP-003: Remove blocking runtime in Clone
│   │   ├── helpers.rs       # GAP-003: Remove block_on patterns
│   │   └── ui_volume.rs     # GAP-016: Refresh strategy
│   ├── client/
│   │   ├── error.rs         # GAP-012: Error context preservation
│   │   └── *.rs             # GAP-006: ClientPool pattern
│   └── ui/
│       └── volumes/
│           └── update/
│               ├── partition.rs   # GAP-010: TODO cleanup
│               └── encryption.rs  # GAP-010: TODO cleanup

storage-dbus/
├── src/
│   └── disks/
│       ├── volume.rs             # GAP-002: Required Connection
│       ├── volume_model/
│       │   ├── partition.rs      # GAP-002: Required Connection
│       │   └── filesystem.rs     # GAP-002: Required Connection
│       └── ops.rs                # GAP-011: Mutex handling

storage-service/
├── src/
│   ├── partitions.rs        # GAP-008: Input validation
│   ├── conversions.rs       # GAP-009: Delete after migration
│   └── main.rs              # GAP-014: Already-running check

storage-models/
└── src/
    └── common.rs            # GAP-005: Remove unwrap in parsing

storage-btrfs/
└── src/
    └── subvolume.rs         # GAP-005: Remove unwrap in path handling

tests/
└── integration/
    └── *.rs                 # GAP-015: New integration tests
```

**Structure Decision**: Existing workspace structure maintained. Changes are internal to crates, no new crates needed.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| ClientPool abstraction | GAP-006 requires shared client ownership | Per-operation client creation causes runtime issues |
| Integration test scaffolding | GAP-015 requires automated contract testing | Manual testing doesn't catch serialization regressions |

## Implementation Phases

### Phase 1: Critical Fixes (P1)

**Goal**: Eliminate all panic paths

1. **GAP-002**: Make Connection required in VolumeNode/VolumeModel
2. **GAP-003**: Remove blocking runtime creation in UiDrive::clone
3. **GAP-004**: Implement parent_path population in flatten_volumes
4. **GAP-005 (partial)**: Fix unwrap in hot paths
5. **GAP-011**: Handle mutex poisoning

### Phase 2: High Priority (P2)

**Goal**: Improve reliability and developer experience

1. **GAP-006**: Implement ClientPool pattern
2. **GAP-008**: Add partition input validation
3. **GAP-012**: Improve error context preservation
4. **GAP-015**: Create integration test scaffolding
5. **GAP-016**: Document refresh strategy

### Phase 3: Polish (P3)

**Goal**: Clean up technical debt

1. **GAP-009**: Delete conversions.rs (if Phase 3A complete)
2. **GAP-010**: Link TODOs to issues
3. **GAP-013**: Add operation timeouts
4. **GAP-014**: Add already-running check

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking existing behavior | Medium | High | Incremental changes with tests |
| GAP-002 refactoring cascade | Medium | Medium | Do incrementally per-file |
| Integration tests need mock UDisks2 | High | Medium | Start with serialization tests only |
| ClientPool changes touch many files | High | Low | Create trait for injection |

## Dependencies

- No external dependencies
- All changes internal to existing crates
- Order matters: GAP-003 depends on GAP-006 pattern being established
