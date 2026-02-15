# Feature Specification: Refactor Build Workflow & UI Analysis

**Feature Branch**: `feature/storage-service`
**Created**: 2026-02-14
**Status**: Draft
**Input**: User description: "for this current branch (continuation of refactoring work): just file default case should: build workspace, install dbus policy & polkit, stop service, start-service-bg, run cosmic-ext-disks. Make sure we've not got any redundant just file logic, propose simplifying if necessary. Analyze disks-ui for overcomplexity, poor conventions, etc. and plan required changes."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Developer Quick-Start Workflow (Priority: P1)

As a developer working on the COSMIC Disks application, I want a single command that sets up and runs the full development environment so that I can start coding without manual multi-step procedures.

**Why this priority**: This is the primary friction point for development velocity. A streamlined default workflow directly impacts daily productivity.

**Independent Test**: Can be fully tested by running `just` (or `just default`) from the project root and verifying that the workspace builds, policies are installed, the service starts in the background, and the UI application launches successfully.

**Acceptance Scenarios**:

1. **Given** a clean repository state, **When** the developer runs `just`, **Then** the workspace builds, D-Bus and Polkit policies are installed, any existing service is stopped, the storage service starts in the background, and the cosmic-ext-disks UI launches.
2. **Given** an already-running service, **When** the developer runs `just`, **Then** the existing service is stopped before starting a fresh instance.

---

### User Story 2 - Justfile Simplification (Priority: P2)

As a developer, I want the justfile to contain only necessary, non-redundant recipes so that I can easily understand available commands and maintain the build system.

**Why this priority**: Reduces cognitive load and maintenance burden, but doesn't block development.

**Independent Test**: Can be tested by reviewing the justfile for duplicate logic and verifying that removed recipes don't break any documented workflows.

**Acceptance Scenarios**:

1. **Given** the current justfile, **When** reviewed for redundancy, **Then** any recipes with overlapping/duplicate build steps are identified and consolidated.
2. **Given** simplified justfile, **When** all documented workflows are executed, **Then** they complete successfully without regression.

---

### User Story 3 - Storage-UI Architecture Analysis (Priority: P3)

As a developer, I want a documented analysis of the storage-ui crate's architecture identifying areas of overcomplexity and convention violations so that future refactoring efforts are well-guided.

**Why this priority**: Provides roadmap for future improvements but doesn't immediately impact functionality.

**Independent Test**: Can be tested by delivering the analysis document and verifying it covers all major modules with actionable recommendations.

**Acceptance Scenarios**:

1. **Given** the storage-ui crate, **When** architecture analysis is performed, **Then** areas of overcomplexity, poor conventions, and improvement opportunities are documented.
2. **Given** the analysis, **When** recommendations are reviewed, **Then** they include specific file locations and concrete refactoring strategies.

---

### Edge Cases

- What happens when the developer doesn't have sudo access for policy installation?
- What happens when the storage service fails to start (e.g., port conflicts)?
- What happens when D-Bus or Polkit installation fails mid-process?
- How does the workflow handle repeated invocations while a previous instance is still running?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The default justfile recipe MUST execute the complete development workflow in sequence: build workspace, install policies, stop existing service, start service in background, launch UI application.
- **FR-002**: The justfile MUST install both D-Bus policy and Polkit policy as part of the default workflow.
- **FR-003**: The justfile MUST stop any existing cosmic-storage-service process before starting a new instance.
- **FR-004**: The justfile MUST start the storage service in background mode (non-blocking) so the UI can launch afterward.
- **FR-005**: The justfile analysis MUST identify redundant build steps across recipes (e.g., repeated `cargo build --workspace` calls).
- **FR-006**: The storage-ui analysis MUST cover the following modules: `ui/app`, `ui/volumes`, `ui/dialogs`, `ui/btrfs`, `ui/sidebar`, `models`, and `client`.
- **FR-007**: The storage-ui analysis MUST identify message routing complexity and propose simplifications.
- **FR-008**: The storage-ui analysis MUST identify state management patterns and assess adherence to COSMIC application conventions.

### Key Entities

- **Justfile Recipe**: A named, executable command in the justfile that performs one or more build/development operations.
- **Storage-UI Module**: A logical grouping of Rust source files handling a specific UI concern (e.g., volumes display, dialogs, BTRFS management).
- **Message Chain**: The pattern of message types flowing from UI widgets through `VolumesControlMessage` to app-level `Message` enum.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developer can execute `just` from a fresh terminal and have a fully running development environment (service + UI) without any additional manual commands.
- **SC-002**: The justfile contains zero redundant build invocations (no repeated `cargo build --workspace` across recipes that can be shared).
- **SC-003**: The storage-ui analysis identifies at least 3 specific improvement opportunities with file paths and line references.
- **SC-004**: All existing justfile workflows (`just dev`, `just start-service-bg`, `just install-dev-policies`) continue to function after refactoring.

## Assumptions

- The developer has sudo privileges for system-level installations (D-Bus/Polkit policies).
- The development environment already has Rust and required system dependencies installed.
- The current `install-dev-policies`, `stop-service`, `start-service-bg`, and `start-app` recipes are functionally correct and can be composed.

## Out of Scope

- Modifying the storage-service backend code.
- Adding new features to the UI.
- Changing the D-Bus or Polkit policy configurations.
- Performance optimization of build times.
