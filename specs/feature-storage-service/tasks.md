# Tasks: Refactor Build Workflow & UI Analysis

**Input**: Design documents from `/specs/feature-storage-service/`
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅

**Tests**: Not explicitly requested. Manual verification of workflows required.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Verification)

**Purpose**: Verify development environment and current state

- [x] T001 Verify cargo workspace builds successfully with `cargo build --workspace`
- [x] T002 Verify existing justfile recipes work: `just build`, `just install-dev-policies`, `just stop-service`, `just start-service-bg`, `just start-app`
- [x] T003 Run quality gates to establish baseline: `cargo test --workspace`, `cargo clippy --workspace`, `cargo fmt --all --check`

---

## Phase 2: Foundational

**Purpose**: No foundational tasks required - this is a refactoring task using existing infrastructure

*Skip to User Story phases*

---

## Phase 3: User Story 1 - Developer Quick-Start Workflow (Priority: P1) 🎯 MVP

**Goal**: Create a default justfile recipe that executes the complete development workflow in one command

**Independent Test**: Run `just` from project root and verify workspace builds, policies install, service starts in background, UI launches

### Implementation for User Story 1

- [x] T004 [US1] Create new `default` recipe in justfile that replaces current `@just --list` with the dev workflow chain
- [x] T005 [US1] Implement default recipe sequence: build → install-dev-policies → stop-service → start-service-bg → start-app in justfile
- [x] T006 [US1] Add error handling for sudo failures in policy installation step in justfile
- [x] T007 [US1] Add helpful status messages between workflow steps in justfile
- [x] T008 [US1] Test default recipe: run `just` and verify complete workflow executes successfully
- [x] T009 [US1] Test idempotency: run `just` twice and verify second run handles already-running service correctly

**Checkpoint**: At this point, `just` should launch the full development environment

---

## Phase 4: User Story 2 - Justfile Simplification (Priority: P2)

**Goal**: Remove redundant build invocations and consolidate duplicate logic

**Independent Test**: Review justfile for duplicate `cargo build` calls, verify all documented workflows still function

### Implementation for User Story 2

- [x] T010 [US2] Audit justfile for duplicate `cargo build --workspace` calls across recipes
- [x] T011 [US2] Refactor `dev` recipe to call `build` recipe instead of inlining cargo command in justfile
- [x] T012 [US2] Refactor `start-service` recipe to depend on `build` recipe instead of inlining in justfile
- [x] T013 [US2] Refactor `start-service-bg` recipe to depend on `build` recipe instead of inlining in justfile
- [x] T014 [US2] Refactor `start-app` recipe to depend on `build` recipe instead of inlining in justfile
- [x] T015 [US2] Remove or consolidate `dev-clean` recipe if redundant with refactored recipes in justfile
- [x] T016 [US2] Verify all existing workflows still work: `just dev`, `just start-service-bg`, `just start-app`
- [x] T017 [US2] Run quality gates: `cargo clippy --workspace`, `cargo fmt --all --check`

**Checkpoint**: At this point, justfile should have zero redundant build invocations

---

## Phase 5: User Story 3 - Storage-UI Architecture Analysis (Priority: P3)

**Goal**: Document analysis findings with specific file locations and actionable recommendations

**Independent Test**: Verify analysis document covers all required modules with concrete refactoring strategies

### Implementation for User Story 3

- [x] T018 [P] [US3] Document message routing analysis in specs/feature-storage-service/research.md (already done - verify completeness)
- [x] T019 [P] [US3] Document state management analysis in specs/feature-storage-service/research.md (already done - verify completeness)
- [x] T020 [P] [US3] Document update handler organization analysis in specs/feature-storage-service/research.md (already done - verify completeness)
- [x] T021 [US3] Add concrete refactoring recommendations with file paths and line numbers to research.md
- [x] T022 [US3] Create prioritized improvement backlog in research.md based on findings
- [x] T023 [US3] Verify FR-006 coverage: analysis covers ui/app, ui/volumes, ui/dialogs, ui/btrfs, ui/sidebar, models, client modules
- [x] T024 [US3] Verify FR-007 coverage: message routing complexity documented with simplification proposals
- [x] T025 [US3] Verify FR-008 coverage: state management patterns and COSMIC convention adherence documented

**Checkpoint**: Analysis document should be complete with actionable recommendations

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final verification and documentation updates

- [x] T026 Run full quality gate suite: `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all --check`
- [x] T027 [P] Update README.md if justfile workflow documentation needs changes
- [x] T028 Verify SC-001: `just` from fresh terminal gives running dev environment
- [x] T029 Verify SC-002: Count and confirm zero redundant build invocations in justfile
- [x] T030 Verify SC-003: Confirm analysis identifies at least 3 improvement opportunities with file paths
- [x] T031 Verify SC-004: Confirm all existing workflows (`just dev`, `just start-service-bg`, `just install-dev-policies`) still function

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - verify baseline first
- **Foundational (Phase 2)**: Skipped - not needed for this refactoring
- **User Story 1 (Phase 3)**: Depends on Setup completion
- **User Story 2 (Phase 4)**: Depends on User Story 1 (modifies overlapping recipes)
- **User Story 3 (Phase 5)**: Independent - can run in parallel with US1/US2
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Independent - can start after Setup
- **User Story 2 (P2)**: Depends on US1 (both modify default recipe area)
- **User Story 3 (P3)**: Independent - documentation only, already partially complete

### Parallel Opportunities

- T018, T019, T020 can run in parallel (different analysis sections)
- US3 can run in parallel with US1 if team capacity allows

---

## Parallel Example: User Story 3

```bash
# These analysis tasks can run in parallel:
Task: "Document message routing analysis in research.md"
Task: "Document state management analysis in research.md"
Task: "Document update handler organization analysis in research.md"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify baseline)
2. Complete Phase 3: User Story 1 (default recipe)
3. **STOP and VALIDATE**: Run `just` and verify complete workflow
4. Deploy/demo if ready - developers immediately benefit

### Incremental Delivery

1. Complete Setup → Baseline established
2. Add User Story 1 → `just` works → Test independently → Deploy (MVP!)
3. Add User Story 2 → No redundancy → Test workflows → Deploy
4. Add User Story 3 → Analysis complete → Review recommendations → Document
5. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- User Story 3 research.md is already partially complete from /speckit.plan
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All justfile changes must preserve backward compatibility with existing recipes
