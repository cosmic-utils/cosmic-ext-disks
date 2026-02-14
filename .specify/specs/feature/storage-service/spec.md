# Feature Specification: Audit 2026-02-14 Gap Remediation

**Feature Branch**: `feature/storage-service`
**Created**: 2026-02-14
**Status**: Draft
**Input**: Audit report `.copi/audits/2026-02-14T17-00-36Z.md`
**Scope**: GAP-002 through GAP-016 (GAP-001 completed)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Eliminate Runtime Panics (Priority: P1) 🎯 MVP

As a user, I want the disk utility to never crash unexpectedly so that I can trust it with my data.

**Why this priority**: Constitution Principle I (Data Safety) is NON-NEGOTIABLE. Panics violate this principle directly.

**Independent Test**: Run all operations on various disk types without any panic/crash.

**Acceptance Scenarios**:

1. **Given** a VolumeNode with connection, **When** any operation is called, **Then** no `unwrap()` panic occurs even if connection state changes
2. **Given** a UiDrive being cloned, **When** clone is called from async context, **Then** no deadlock or runtime creation panic occurs
3. **Given** any code path in storage-ui/storage-service, **When** unexpected conditions arise, **Then** errors are propagated gracefully, not panicked

---

### User Story 2 - Correct Volume Hierarchy Display (Priority: P1)

As a user, I want to see LUKS containers, LVM volumes, and BTRFS subvolumes correctly nested in the UI tree.

**Why this priority**: UI tree construction is broken for advanced volume types, affecting usability.

**Independent Test**: Create LUKS container, unlock it, verify cleartext device appears as child in tree.

**Acceptance Scenarios**:

1. **Given** a LUKS encrypted partition, **When** unlocked, **Then** the cleartext device shows `parent_path` pointing to the encrypted container
2. **Given** an LVM physical volume with logical volumes, **When** displayed in UI, **Then** LVs appear nested under the PV
3. **Given** BTRFS subvolumes, **When** displayed, **Then** parent-child relationships are correct

---

### User Story 3 - Clear Error Messages (Priority: P2)

As a user, I want clear error messages when operations fail so I understand what went wrong and how to fix it.

**Why this priority**: User experience depends on actionable feedback.

**Independent Test**: Attempt invalid partition creation, verify clear validation error (not cryptic UDisks2 error).

**Acceptance Scenarios**:

1. **Given** partition creation with size=0, **When** submitted, **Then** error message says "Partition size must be greater than zero"
2. **Given** partition creation with misaligned offset, **When** submitted, **Then** error message explains alignment requirements
3. **Given** D-Bus method failure, **When** error reaches UI, **Then** D-Bus error name is preserved in logs and user sees actionable message

---

### User Story 4 - Consistent Client Architecture (Priority: P2)

As a developer, I want a clear client ownership model so that the codebase is maintainable and testable.

**Why this priority**: Technical debt from inconsistent patterns slows development.

**Independent Test**: Verify single D-Bus connection shared across all UiDrive instances.

**Acceptance Scenarios**:

1. **Given** AppModel startup, **When** clients are created, **Then** a single ClientPool is shared via Arc
2. **Given** any UiDrive operation, **When** client access needed, **Then** it uses shared clients, not per-operation creation
3. **Given** tests, **When** mocking is needed, **Then** mock clients can be injected via ClientPool trait

---

### User Story 5 - Operation Timeouts and Progress (Priority: P3)

As a user, I want to see progress during long operations and have them timeout if stuck.

**Why this priority**: UX improvement for slow operations.

**Independent Test**: Format a large disk, verify progress bar updates.

**Acceptance Scenarios**:

1. **Given** format operation starting, **When** progress signals received, **Then** UI progress bar updates
2. **Given** any operation, **When** it exceeds timeout (10 min default), **Then** operation is cancelled with clear message
3. **Given** restore image operation, **When** in progress, **Then** user sees percentage complete

---

### User Story 6 - Integration Test Coverage (Priority: P2)

As a developer, I want automated integration tests so refactors don't silently break the client-service contract.

**Why this priority**: Prevents regression during ongoing refactor.

**Independent Test**: Run `cargo test --workspace --all-features` and verify integration tests pass.

**Acceptance Scenarios**:

1. **Given** storage-service running, **When** client calls list_disks, **Then** response deserializes correctly
2. **Given** serialization round-trip test, **When** DiskInfo is serialized/deserialized, **Then** all fields match
3. **Given** partition creation, **When** called without auth, **Then** PermissionDenied error returned

---

### User Story 7 - Clean Up Tech Debt (Priority: P3)

As a developer, I want temporary code removed and TODOs resolved so the codebase is maintainable.

**Why this priority**: Reduces maintenance burden.

**Independent Test**: Grep for conversions.rs, orphaned TODOs.

**Acceptance Scenarios**:

1. **Given** conversions.rs exists, **When** Phase 3A is complete, **Then** file is deleted
2. **Given** TODO comments in production code, **When** audited, **Then** each is linked to issue or marked DEFERRED
3. **Given** service already running, **When** second instance starts, **Then** clear error message displayed

---

### Edge Cases

- What happens when D-Bus connection is lost mid-operation?
- How does system behave with 100+ concurrent volume refreshes?
- What if Polkit dialog is cancelled during authorization?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST NOT panic in any production code path (Constitution I)
- **FR-002**: All VolumeNode/VolumeModel operations MUST handle connection as required field or proper Option
- **FR-003**: UiDrive clone MUST NOT create blocking runtimes (GAP-003)
- **FR-004**: VolumeInfo.parent_path MUST be populated during tree flattening (GAP-004)
- **FR-005**: Partition operations MUST validate inputs before calling UDisks2 (GAP-008)
- **FR-006**: Client lifecycle MUST be documented and consistent (GAP-006)
- **FR-007**: Long operations MUST have configurable timeouts (GAP-013)
- **FR-008**: Integration tests MUST cover D-Bus serialization contracts (GAP-015)
- **FR-009**: Error conversions MUST preserve D-Bus error names (GAP-012)
- **FR-010**: Mutex poisoning MUST be handled gracefully (GAP-011)

### Key Entities

- **VolumeNode**: Storage hierarchy node; MUST have required Connection, not Optional
- **UiDrive**: UI model for disk; MUST use shared clients via Arc, not owned clients
- **ClientPool**: Centralized D-Bus client management; shared across all UI components
- **RefreshResult**: Atomic refresh outcome; includes NotFound variant for stale tree detection

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero `unwrap()` or `panic!()` in production code paths (except test code)
- **SC-002**: Zero `Runtime::new()` or `block_on()` in storage-ui
- **SC-003**: All VolumeInfo objects have correct parent_path populated
- **SC-004**: grep `unwrap\(` shows <20 hits in non-test code
- **SC-005**: Integration tests for all D-Bus interfaces pass
- **SC-006**: ClientPool pattern used consistently in storage-ui
- **SC-007**: All TODOs linked to issues or marked DEFERRED

## Gap Mapping

| Gap ID | Severity | User Story | Priority |
|--------|----------|------------|----------|
| GAP-002 | CRITICAL | US1 - Eliminate Panics | P1 |
| GAP-003 | CRITICAL | US1 - Eliminate Panics | P1 |
| GAP-004 | HIGH | US2 - Volume Hierarchy | P1 |
| GAP-005 | HIGH | US1 - Eliminate Panics | P1 |
| GAP-006 | HIGH | US4 - Client Architecture | P2 |
| GAP-007 | MEDIUM | US6 - Integration Tests | P2 |
| GAP-008 | MEDIUM | US3 - Clear Errors | P2 |
| GAP-009 | MEDIUM | US7 - Tech Debt | P3 |
| GAP-010 | LOW | US7 - Tech Debt | P3 |
| GAP-011 | MEDIUM | US1 - Eliminate Panics | P1 |
| GAP-012 | LOW | US3 - Clear Errors | P2 |
| GAP-013 | MEDIUM | US5 - Timeouts/Progress | P3 |
| GAP-014 | LOW | US7 - Tech Debt | P3 |
| GAP-015 | MEDIUM | US6 - Integration Tests | P2 |
| GAP-016 | MEDIUM | US4 - Client Architecture | P2 |
