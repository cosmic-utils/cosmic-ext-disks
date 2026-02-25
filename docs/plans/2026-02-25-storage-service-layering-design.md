# Storage Service Layering Design (Transport/Domain/Adapters)

**Date:** 2026-02-25  
**Branch:** `069-polish`

## Goal

Remove conceptual overlap between handlers and domain by introducing explicit layer names and ownership:

- `transport` for D-Bus interface handling
- `domain` for policy/normalization/orchestration
- `adapters` for system I/O boundaries

## Problem Statement

Current `storage-service` structure mixes concerns across:

- root-level handler modules (`btrfs.rs`, `disks.rs`, etc.)
- `service/domain/*` policy modules (`Default*Domain` + `*Domain` traits)
- `adapters/*` boundary modules

The layering exists functionally, but names and placement make it look like competing abstractions.

## Architecture Decision

Adopt a 3-layer dependency direction:

`transport -> domain -> adapters`

### Layer definitions

1. **transport**
   - Owns D-Bus method signatures, signal emission, marshaling, and transport-level error mapping.
   - May enforce transport concerns (auth macros and call context), but not business policy.

2. **domain**
   - Owns capability checks, validation, normalization, and use-case orchestration.
   - Pure service logic; no D-Bus-specific types.

3. **adapters**
   - Owns concrete system and external tool interactions.
   - Exposes trait boundaries for storage operations/query behavior.

## Target Module Map

### Before

- `src/{btrfs,disks,filesystems,image,luks,lvm,partitions,rclone}.rs` (handlers + some policy wiring)
- `src/service/domain/*` (domain traits/default impls)
- `src/adapters/*`
- `src/service.rs` (top-level service interface)

### After

- `src/transport/{btrfs,disks,filesystems,image,luks,lvm,partitions,rclone}.rs`
- `src/transport/service.rs`
- `src/domain/{btrfs,disks,filesystems,image,luks,lvm,partitions,rclone}.rs`
- `src/adapters/*` (unchanged placement)
- root keeps cross-cutting modules (`auth.rs`, `error.rs`, `routing.rs`, `protected_paths.rs`, `main.rs`)

## Ownership Rules (No Competing Ideas)

- `transport` does not implement policy or capability logic.
- `domain` does not emit D-Bus signals or carry transport-specific concerns.
- `adapters` do not own business rules or D-Bus formatting.
- Cross-layer calls must follow `transport -> domain -> adapters` only.

## Migration Strategy

### Phase 1: Structure-only move (no behavior changes)

- Create `src/transport` and `src/domain`.
- Move existing modules to new locations.
- Update `mod` declarations and import paths.
- Keep all type/function names and behavior unchanged.

### Phase 2: Boundary hardening

- Move remaining policy checks from transport handlers into domain modules where needed.
- Keep adapter contracts stable.
- Ensure transport files remain thin and D-Bus-focused.

### Phase 3 (Optional): Naming cleanup

- Rename `Default*Domain` to intention-revealing names (e.g., `*Policy`/`*Service`) if desired.
- Keep this separate from structural migration to reduce risk.

## Non-Goals

- No D-Bus object path/interface name changes.
- No adapter implementation behavior changes.
- No feature set expansion.

## Risks and Mitigations

- **Risk:** Large import churn during moves.  
  **Mitigation:** Phase 1 is move-only with compile checks at each step.

- **Risk:** Accidental behavior drift while reorganizing.  
  **Mitigation:** No logic edits in Phase 1; defer behavior shifts to Phase 2.

- **Risk:** Refactor size obscures regressions.  
  **Mitigation:** Keep phase-scoped commits and verify after each phase.

## Verification Gates

Run after each phase:

- `cargo check -p storage-service`
- `cargo test -p storage-service --no-run`
- `just verify`

## Acceptance Criteria

- Clear module placement for `transport`, `domain`, and `adapters`.
- No ambiguous “handler vs domain” ownership left in architecture docs/code layout.
- Existing D-Bus clients remain compatible.
- Workspace verification commands pass.