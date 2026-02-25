# Storage Service Adapter Strip-Out Design

**Date:** 2026-02-25  
**Branch:** `069-polish`

## Goal

Simplify `storage-service` by removing the adapter indirection layer and routing registry, and calling `storage-udisks` directly from handlers, consistent with current direct usage of `disks_btrfs` and `storage_sys`.

## Problem Statement

`storage-service` currently routes core concerns (disks/partitions/filesystems/luks/image) through:

- `adapters/udisks.rs` trait-object wrappers
- `routing.rs` registry and concern routing table
- `storage-contracts` adapter traits used by handlers

This adds indirection and conceptual overhead without active backend swapping.

## Decision

Adopt direct backend calls for UDisks concerns:

- `handlers/*` call `storage-udisks` APIs directly
- Keep `policies/*` for business validation/capability logic
- Remove adapter and routing infrastructure from `storage-service`

## Target Architecture

- `handlers/` — D-Bus transport methods/signals + orchestration entrypoints
- `policies/` — normalization/validation/capability checks
- Direct backend libraries:
  - `storage-udisks` for disk/partition/filesystem/luks/image concerns
  - `disks_btrfs` for BTRFS concerns
  - `storage_sys` for system/rclone/image-io support concerns

Dependency shape:

`handlers -> policies` and `handlers -> backend crates`

## Scope of Removal

Remove from `storage-service`:

- `src/adapters/mod.rs`
- `src/adapters/udisks.rs`
- `src/routing.rs`

Rewire in `src/main.rs`:

- Remove `AdapterRegistry` creation and route logging
- Construct handlers directly (no adapter trait injection)

Refactor handlers:

- `DisksHandler`
- `PartitionsHandler`
- `FilesystemsHandler`
- `LuksHandler`
- `ImageHandler`

These handlers stop importing `storage-contracts` adapter traits and use `storage-udisks` directly.

## Non-Goals

- No D-Bus interface/path/signal changes
- No auth/polkit behavior changes
- No feature-flag behavior changes
- No functional expansion

## Migration Plan (High-Level)

### Phase 1: Direct-call cutover

- Update handler constructors and internal call sites to use `storage-udisks` directly.
- Keep behavior identical; only replace call path.

### Phase 2: Remove adapter/routing infrastructure

- Delete adapter and routing modules.
- Remove module declarations/imports and dead route logging.

### Phase 3: Contracts cleanup (if now unused)

- Remove adapter-trait exports/definitions from `storage-contracts` if no remaining users.
- Preserve protocol/shared contracts still used across crates.

## Risks and Mitigations

- **Risk:** Signature mismatches when replacing trait calls with concrete calls.  
  **Mitigation:** Small, concern-by-concern replacements with compile checks.

- **Risk:** Behavior drift in error mapping or defaults.  
  **Mitigation:** Preserve existing error text and control flow where possible.

- **Risk:** Hidden cross-crate dependence on adapter traits.  
  **Mitigation:** workspace grep + full verify before trait removal.

## Verification Gates

Run after each phase:

- `cargo check -p storage-service`
- `cargo test -p storage-service --no-run`
- `just verify`

## Acceptance Criteria

- No `adapters/` or `routing.rs` in `storage-service`.
- No adapter trait usage in `storage-service` handlers.
- Handlers call `storage-udisks` directly for UDisks-backed concerns.
- D-Bus compatibility preserved.
- Verification gates pass.
