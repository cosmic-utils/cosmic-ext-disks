# Layered Crate Structure Design (Workspace-Wide)

**Date:** 2026-02-26  
**Status:** Approved in brainstorming session  
**Scope:** Workspace crate structure normalization and aggressive renames, with paradigm preservation for `storage-app` and `storage-service`.

---

## 1. Goals

1. Improve onboarding clarity by making crate navigation predictable.
2. Increase layer isolation and reduce accidental cross-layer coupling.
3. Apply aggressive naming/path cleanup where it improves consistency.
4. Preserve intentionally layered paradigms in `storage-app` and `storage-service`.

## 2. Non-Goals

1. No paradigm rewrite of `storage-app` away from its current `state/message/view/control/update` split.
2. No paradigm rewrite of `storage-service` away from its current `handlers/policies/utilities` style.
3. No feature additions, behavior changes, or API expansion beyond what is needed for structural consistency.

## 3. Architecture Decision

Adopt **layer-first structure** as the workspace default, with domain names as secondary grouping where needed:

- `model` (data models / DTOs)
- `api` (public entry points / stable façade)
- `ops` (business or operation logic)
- `infra` (DBus/syscall/CLI/process adapters)
- `error` (crate-local error surface)

### Exceptions (explicit and intentional)

- `storage-app` remains the reference layered UI architecture with existing split modules.
- `storage-service` remains the reference service layered architecture with existing split modules.

## 4. Dependency Direction

- `storage-types` is foundational and has no internal crate dependencies.
- `storage-contracts` depends on `storage-types`.
- Implementation crates (`storage-udisks`, `storage-sys`, `storage-btrfs`) depend inward.
- `storage-service` composes implementation crates and exposes DBus interfaces.
- `storage-app` sits at the edge and consumes service/client contracts + types.

No back-edges across layers are allowed.

## 5. Crate-by-Crate Target Structure

## 5.1 `storage-app` (preserve paradigm; improve only)

Keep current top-level shape:

- `controls/`
- `errors/`
- `message/`
- `state/`
- `subscriptions/`
- `update/`
- `views/`
- `client/`, `models/`, `utils/`

Improvements only:

- Dissolve top-level feature silos `src/volumes` and `src/network`.
- Ensure feature-heavy paths map back into `message/state/update/views/controls/utils` rather than creating parallel architecture trees.
- Normalize naming and ownership so helper code lives with the layer that owns it.

### 5.1.1 `storage-app` finalized move map

- `src/volumes/disk_header.rs` → `src/views/disk.rs`
- `src/volumes/usage_pie.rs` → `src/controls/usage_pie.rs`
- `src/volumes/helpers.rs` split:
   - volume tree + segment lookup helpers (`find_volume_in_ui_tree`, `find_volume_for_partition`, `find_segment_for_volume`) → `src/state/volumes.rs`
   - BTRFS detection helpers (`detect_btrfs_in_node`, `detect_btrfs_for_volume`) → `src/state/btrfs.rs`
   - update operation helper (`collect_mounted_descendants_leaf_first`) → `src/update/volumes/helpers.rs`
   - partition type mapping helpers (`common_partition_filesystem_type`, `common_partition_type_index_for`) → `src/utils/partition_types.rs`
- `src/network/icons.rs` + network scope badge helpers from `views/network.rs` (`scope_icon`, `scope_label`) → `src/controls/icons.rs` (centralized app icon mapping)
- consolidate duplicated disk usage aggregation logic (LUKS child rollup) into one helper under `src/state/volumes.rs`, consumed by `views/app.rs` and `views/disk.rs`
- remove obsolete modules:
   - `src/volumes/mod.rs`
   - `src/network/mod.rs`
   - `mod volumes;` and `mod network;` in `main.rs`

### 5.1.2 `storage-app` image update module decision

- Keep folder form (`src/update/image/`) and remove redundant wrappers.
- Convert `src/update/image.rs` into `src/update/image/mod.rs`.
- Keep dialog lifecycle in `src/update/image/dialogs.rs`.
- Keep operation/start logic in `src/update/image/ops.rs`.
- Update `src/update/mod.rs` to call final image module API directly and remove local wrapper passthroughs.
- Deduplicate repeated `ImageOperationDialog` construction with one local builder/helper in `src/update/image/mod.rs`.

## 5.2 `storage-service` (preserve paradigm; improve only)

Keep current top-level shape:

- `handlers/`
- `policies/`
- `utilities/`
- root `main.rs` composition bootstrap

Improvements only:

- Keep a strict separation: handlers orchestrate, policies authorize, utilities perform reusable infra/system helpers.
- Align naming across handler/policy pairs where mismatch exists.
- Remove utility-layer dependency on handlers by moving hotplug signal logic under disk handler ownership.
- Split oversized handlers (`filesystem`, `rclone`) into folder modules by responsibility while preserving DBus interface paths and names.
- Remove duplicated rclone caller/auth plumbing where macro-injected caller context already exists.

## 5.3 `storage-types`

Target:

- keep public module names stable (`partition_types`, `rclone`) and avoid rename churn
- split oversized modules into folder modules with focused internals
- tighten crate-root exports in `lib.rs` from broad wildcard re-exports to explicit grouped exports

Planned internal structure:

- `src/partition_types/mod.rs` (public façade)
- `src/partition_types/catalog.rs` (embedded TOML catalogs + static data)
- `src/partition_types/query.rs` (lookup/query helpers)
- `src/rclone/mod.rs` (public façade)
- `src/rclone/scope.rs`, `remote.rs`, `provider_catalog.rs`, `mount.rs`, `results.rs`

## 5.4 `storage-contracts`

Target:

- keep `protocol/` + `traits/` layering with singular protocol module names (`error`, `id`, `operation`)
- remove trait naming ambiguity by renaming `FilesystemOps` (discovery) to `FilesystemDiscovery`
- replace crate-root wildcard façade exports with explicit grouped exports for protocol and trait surfaces
- keep protocol payload semantics unchanged (`StorageError*`, `Operation*`)

## 5.5 `storage-udisks`

Target:

- separate operation logic from DBus/process/system integration details
- keep domain logic free from infra-specific implementation leakage
- expose stable crate façade from `lib.rs`

Planned internal structure:

- `src/infra/mod.rs` (private infra façade)
- `src/infra/options.rs`
- `src/infra/udisks_block_config.rs`
- `src/infra/usage.rs`
- `src/infra/process.rs`

Constraints:

- keep public helpers available from crate root via re-exports in `lib.rs`
- remove public `util` module exposure after moving `util/process.rs` into `infra/process.rs`
- keep `src/dbus/**` unchanged as DBus adapter boundary

## 5.6 `storage-sys`

Target:

- keep current top-level module layout for `error`, `image`, and `usage` (no blanket `ops` umbrella)
- split oversized `rclone` module into focused internal submodules under `src/rclone/`
- expose `usage` helpers through explicit façade re-exports to reduce deep module-path coupling
- keep binaries (`src/bin`) thin and dependent on public façade only

## 5.7 `storage-btrfs`

Target:

- maintain compact crate while relying on centralized shared models in `storage-types`
- keep CLI-specific behavior isolated from core library
- remove dead duplicate local model definitions
- avoid exposing low-level `btrfsutil` crate types through public re-exports

## 5.8 `storage-macros`

Target:

- split macro internals (`parse`, `transform`, `emit`) while keeping external macro API stable
- preserve caller parameter detection behavior (`#[zbus(connection)]` / `#[zbus(header)]` and fallback names `connection|_connection`, `header|_header`)
- preserve current compile error text for async-only and required-parameter validation

## 6. Rename Rules

1. Prefer consistent singular/plural conventions inside each crate.
2. Avoid root-level `helpers`, `common`, `utilities` unless they are truly layer-specific; otherwise scope them.
3. Rename for clarity only when net navigation clarity improves.
4. During migration, do not add temporary compatibility re-exports or shim layers; apply final target paths directly.

## 7. Error Handling Rules

1. Keep crate-local error enums local.
2. Convert errors at layer boundaries.
3. Avoid leaking infra-specific error details across crate API boundaries.

## 8. Validation Strategy

Wave validation:

1. Types/contracts wave
2. Backend implementation wave (`storage-udisks`, `storage-sys`, `storage-btrfs`)
3. Service wave
4. App compatibility wave (touch-ups only)

At each wave boundary:

- run `just check`
- ensure workspace compiles
- ensure no dependency-direction regressions

## 9. Acceptance Criteria

1. All crates follow either:
   - the standardized layer-first structure, or
   - approved preserved paradigm (`storage-app`, `storage-service`).
2. No structural anti-pattern folders remain without explicit rationale.
3. Public crate façades are clear and documented through module exports.
4. Final workspace passes `just check`.
