<!--
SYNC IMPACT REPORT
==================
Version change: 1.0.0 → 1.1.0
Modified principles: None
Added sections:
  - Spec & Architecture Documentation
Removed sections: None
Templates requiring updates:
  - .specify/templates/plan-template.md: ✅ No changes needed (generic template)
  - .specify/templates/spec-template.md: ✅ No changes needed (generic template)
  - .specify/templates/tasks-template.md: ✅ No changes needed (generic template)
New files created:
  - .specify/memory/architecture.md (project context for speckit)
  - .specify/specs/ (directory for branch specs)
Follow-up TODOs: None
==================
-->

# COSMIC Disks Constitution

## Core Principles

### I. Data Safety First (NON-NEGOTIABLE)

Disk operations can cause irreversible data loss. All code MUST prioritize data integrity:

- Destructive operations (format, delete partition, etc.) MUST require explicit user confirmation
- Operations MUST validate preconditions before execution (e.g., check if partition is mounted before deletion)
- Error handling MUST be comprehensive—no `unwrap()` or `panic!()` in production code paths
- User-facing warnings MUST clearly communicate risks, especially for beta/untested scenarios
- Partial failures MUST leave the system in a consistent state or provide clear recovery guidance

**Rationale**: Users trust this software with their data. A single bug can destroy irreplaceable files. Safety cannot be compromised for convenience.

### II. Modular Crate Architecture

The workspace is organized into focused crates with clear responsibilities:

| Crate | Purpose |
|-------|---------|
| `storage-ui` | COSMIC GUI application (libcosmic-based) |
| `storage-dbus` | UDisks2 D-Bus abstraction layer |
| `storage-service` | Background D-Bus service for privileged operations |
| `storage-models` | Shared data models and types |
| `storage-sys` | Low-level system operations (commands, sysfs) |
| `storage-btrfs` | BTRFS-specific utilities (subvolumes, snapshots) |

- Each crate MUST have a single, well-defined purpose
- Dependencies between crates MUST be unidirectional (no circular dependencies)
- Shared code goes in `storage-models`; implementation details stay in specific crates
- Module organization: sibling files for small modules (≤3 files), folder with `mod.rs` for larger hierarchies

**Rationale**: Clear boundaries enable independent testing, reduce coupling, and make the codebase navigable for contributors.

### III. Quality Gates (NON-NEGOTIABLE)

All code changes MUST pass the following gates before merge:

- `cargo test --workspace --all-features` MUST pass
- `cargo clippy --workspace --all-features` MUST pass with no warnings
- `cargo fmt --all --check` MUST pass

Additionally:

- New features MUST include appropriate error handling
- Breaking changes to public APIs MUST be documented and versioned appropriately
- Code MUST NOT introduce new `unwrap()`, `expect()`, or `panic!()` in user-facing paths

**Rationale**: Automated quality gates catch regressions early and maintain code quality without requiring manual review of formatting or basic correctness.

### IV. Evidence-Based Design

Architecture and implementation decisions MUST be documented with evidence:

- Design documents MUST cite specific files, functions, or external references supporting decisions
- Spec documents MUST reference the source of requirements (issue, user request, audit finding)
- Code comments SHOULD explain "why" when the reasoning is non-obvious
- Architecture decisions (ADR-style) SHOULD be recorded for significant changes

**Rationale**: Disk management involves complex system interactions. Evidence-based documentation enables future contributors to understand context and avoid repeating mistakes.

### V. Linux System Integration

This application targets Linux desktop environments with deep system integration:

- Target platform: Linux only (systemd-based distributions)
- Primary interface: UDisks2 via D-Bus for storage operations
- Privilege escalation: Polkit for authorized operations
- Required system packages: `udisks2`, `ntfs-3g`, `exfatprogs`, `dosfstools`
- Desktop integration: COSMIC/libcosmic UI framework, XDG portals

Code MUST:

- Use D-Bus/UDisks2 APIs for storage operations (not direct sysfs/kernel interfaces unless necessary)
- Handle D-Bus connection failures gracefully
- Respect system locale via i18n (Fluent)

**Rationale**: Deep system integration provides a native experience but requires respecting Linux desktop conventions and system service dependencies.

## Spec & Architecture Documentation

### Spec Location

Feature specifications MUST be stored in `.specify/specs/{BRANCH_NAME}/`:

```
.specify/specs/
├── feature/
│   ├── storage-service/
│   │   ├── spec.md
│   │   ├── plan.md
│   │   ├── research.md
│   │   └── tasks.md
│   └── btrfs-tools/
│       └── ...
└── fix/
    └── audit-2026-02-14-gaps/
        └── ...
```

- Branch `feature/storage-service` → `.specify/specs/feature/storage-service/`
- Branch `fix/parent-path` → `.specify/specs/fix/parent-path/`
- Branch `chore/update-deps` → `.specify/specs/chore/update-deps/`

### Architecture Overview

The project maintains an architecture overview at `.specify/memory/architecture.md`:

- Provides project context for speckit workflows
- Documents workspace structure and crate responsibilities
- Describes architecture layers and data flow
- Lists key technologies and D-Bus interfaces
- MUST be updated when significant architectural changes are made

**Rationale**: Centralized architecture documentation enables AI assistants and contributors to quickly understand project structure without scanning the entire codebase.

## Technology Constraints

**Language**: Rust (stable channel, edition 2024)
**Minimum Rust Version**: Unpinned (use latest stable)
**Target OS**: Linux only
**UI Framework**: libcosmic (COSMIC desktop toolkit)
**Build System**: Cargo with workspace
**Task Runner**: `just` (see `justfile`)

**Key Dependencies**:
- `udisks2` crate for UDisks2 D-Bus bindings
- `zbus` for low-level D-Bus communication
- `libcosmic` for UI components
- `tokio` for async runtime

## Development Workflow

### Branching

- **Default branch**: `main`
- **Branch naming**: `feature/{slug}`, `fix/{slug}`, `chore/{slug}`
- **Merge strategy**: Squash merge

### Commits

- **Format**: Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `test:`)
- **Scope**: Optional but encouraged for multi-crate changes (e.g., `feat(ui):`, `fix(dbus):`)

### Pull Requests

- CI gates MUST pass (test, clippy, fmt)
- Breaking changes MUST be documented in PR description
- PRs SHOULD reference related specs in `.specify/specs/{branch}/` when applicable

### Versioning

- SemVer with `v` tag prefix (e.g., `v1.2.3`)
- Version bumps via tags/releases, not inferred from commit messages
- Crates are published to crates.io on push to `main`

## Governance

This constitution is authoritative for all development in this repository.

**Amendment Process**:
1. Propose amendment via issue or PR
2. Document rationale and impact on existing code
3. Increment version per semantic versioning rules
4. Update dependent templates and documentation

**Compliance**:
- All PRs MUST verify compliance with this constitution
- Complexity beyond these guidelines MUST be justified in the PR/plan
- When in doubt, prioritize data safety over convenience

**Version**: 1.1.0 | **Ratified**: 2026-02-14 | **Last Amended**: 2026-02-14
