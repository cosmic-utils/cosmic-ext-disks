# Workspace Cleanup and Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Execute a deep, phased refactor of the full workspace to improve architecture, naming, resource organization, and error handling, with no backward-compatibility constraints.

**Architecture:** Apply controlled breaking changes in phased PRs by crate boundary: normalize workspace conventions first, then split shared contracts/types, then refactor service/system/transport/ui layers, finishing with btrfs/macro polish. Keep `storage-service` orchestration-only and trait-driven; tools/adapters remain isolated by external dependency boundary.

**Boundary Rule:** All tool crates (`storage-sys`, `storage-udisks`, `storage-btrfs`, future adapters) are contract-bound adapters and must expose/return only `storage-types` models or `storage-contracts` contract types at their public boundaries.

**App Dependency Rule:** `storage-app` depends only on `storage-types` and, if required, `storage-contracts`; it must not depend on `storage-service` or any tool adapter crate.

**App/Service Isolation Rule:** `storage-app` and `storage-service` have no direct dependency on each other; both depend only on shared crates and their own layer-specific dependencies.

**Protocol Contract Rule:** App/service communication request/response contracts (including DBus-facing contract DTOs) live in `storage-contracts` alongside trait contracts.

**Contracts Module Rule:** `storage-contracts` is organized into `traits` and `protocol` submodules (not a flat namespace).

**Traits Layout Rule:** Organize `storage-contracts::traits` as separate files per concern (not grouped multi-concern files).

**Contracts Import Rule:** Provide `mod.rs` re-exports for ergonomic imports from `storage-contracts::traits` and `storage-contracts::protocol`.

**Protocol Versioning Rule:** Keep `storage-contracts::protocol` unversioned for now; add explicit version namespaces only when needed.

**Protocol Shape Rule:** Use narrow per-concern request/response structs in `storage-contracts::protocol` rather than a single global command/result enum surface.

**Protocol Layout Rule:** Organize `storage-contracts::protocol` as separate files per concern.

**Progress/Event Rule:** Define long-running operation progress/events as first-class types in `storage-contracts::protocol` using enum variants with typed struct payloads. Include rich operation metrics (for example `bytes_done`/`bytes_total`) where relevant, while keeping payload names domain-centric rather than transport-specific.

**Trait Signature Rule:** Use strongly typed request/response structs for non-trivial operations; allow primitive request/response signatures for simple operations, decided case-by-case by engineering judgment.

**Serde Scope Rule:** In `storage-types`, derive serde only for DTOs that cross process/IO boundaries; keep internal-only structs non-serialized.

**Serde Naming Rule:** Use implicit default serde naming for protocol/contracts unless a specific type requires an explicit rename override.

**Enum Serialization Rule:** Use default serde enum representation (externally tagged) for protocol enums unless a specific case requires override.

**Safety Rule:** Enforce `#![forbid(unsafe_code)]` in `storage-types`, `storage-contracts`, `storage-service`, and `storage-app`.

**Adapter Unsafe Rule:** In tool adapter crates (`storage-sys`, `storage-udisks`, `storage-btrfs`), allow `unsafe` only where strictly required by external/sys bindings or APIs; otherwise keep code safe.

**Operation ID Rule:** Use a typed `OperationId` newtype in `storage-types` instead of opaque string IDs.

**Operation ID Backing Rule:** Back `OperationId` with `uuid::Uuid`.

**Correlation Rule:** Operation correlation IDs are required across `storage-service` and all tool adapters (`storage-udisks`, `storage-sys`, `storage-btrfs`) for logs and progress/event emission.

**Error Taxonomy Rule:** `StorageErrorKind` is a closed enum (no `Other(String)` escape hatch).

**Error Code Rule:** `StorageErrorKind` remains the single error taxonomy. Define numeric machine codes via a method on the enum (for example `code()`), grouping categories in hundreds (e.g., 100s validation, 200s permissions, 300s transport, etc.).

**Error Code Stability Rule:** Aim for stable error codes across releases, while allowing deliberate updates during alpha when taxonomy improvements require changes.

**Concern Split Rule:** Procedure-specific validation and preconditions tied to an adapter API live in that adapter crate (`storage-udisks`, `storage-sys`, `storage-btrfs`, and future adapters). Cross-tool sequencing, routing, retries, and global operation orchestration live in `storage-service`.

**Module Naming Rule:** Enforce strict concern-oriented module naming workspace-wide. Avoid generic `misc`/`helpers`/`utils` modules except for truly cross-cutting shared utilities.

**Shared Utility Rule:** Centralize truly cross-cutting utility code in shared crates/modules (instead of duplicating helpers per concern).

**Public API Surface Rule:** Keep `storage-types` and `storage-contracts` public surfaces minimal; export only symbols required by dependent crates.

**Test Layout Rule:** Prefer centralized in-crate `tests/` folders where possible, rather than scattering tests inline across many modules.

**Migration Script Rule:** Do not retain temporary migration/rewrite scripts after the refactor; use one-off commands only.

**Formatting Rule:** Use rustfmt defaults only (`cargo fmt`); do not introduce additional custom formatting configuration for this refactor.

**Lint Rule:** Keep the current clippy baseline (`cargo clippy --workspace --all-targets`) without introducing additional deny-list expansions as part of this refactor.

**Dependency Hygiene Rule:** Apply strict dependency/feature pruning in each touched crate; remove all unused dependencies/features immediately when identified.

**Tech Stack:** Rust workspace (`storage-types`, `storage-contracts`, `storage-sys`, `storage-udisks` (renamed from `storage-dbus`), `storage-service`, `storage-macros`, `storage-btrfs`, `storage-app` (renamed from `storage-ui`)), libcosmic/iced, zbus, serde, tracing, cargo fmt/clippy/test.

**Naming Rule:** Only app-level crates (`storage-app`, `storage-service`) use the `cosmic-ext-` prefix in package naming. Internal/shared/tool crates (`storage-types`, `storage-contracts`, `storage-sys`, `storage-udisks`, `storage-btrfs`, macros) keep neutral package naming without the `cosmic-ext-` prefix.

**App Identity Rule:** The app package name and end-user binary name remain `cosmic-ext-storage` even though the crate directory/module naming is renamed from `storage-ui` to `storage-app`.

**Rename Scope Rule:** Apply full rename propagation for approved crate renames across code, manifests, docs, scripts, CI/task configs, and resource references. Preserve public app identity artifacts (`cosmic-ext-storage` package/binary/app-id paths) where explicitly required.

**Execution Order Rule:** Perform structural crate renames first (`storage-ui`/`storage-dbus`/`storage-service-macros`), then execute the `storage-common` split into `storage-types` + `storage-contracts`.

**History Preservation Rule:** During `storage-common` split/removal, preserve file history with moves (`git mv`) where practical before content edits.

**Split Phase Commit Rule:** Use a single commit for the `storage-common` -> (`storage-types`, `storage-contracts`) split/removal phase.

**Execution Branch Rule:** Execute this plan on the current branch (`069-polish`).

**Documentation Strategy Rule:** Keep existing design/plan docs and append/update incrementally rather than replacing with a single consolidated document.

**Rename Phase Rule:** Execute all approved renames in one single rename mega-phase before contract/type split and behavioral refactors.

**Rename Phase Scope Rule:** Rename mega-phase is strictly rename/move/reference propagation only with zero logic/behavior changes.

**Rename Phase Commit Rule:** Use a single commit for the rename mega-phase.

**Rename Phase Build Rule:** Temporary non-compiling state is allowed during rename mega-phase; full workspace compile must be restored at the next phase boundary.

**Post-Rename Validation Rule:** When restoring build after rename mega-phase, require full baseline validation (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace --no-run`).

**Technical Debt Marker Rule:** Remove all `TODO`/`FIXME` markers from code during the refactor. Any intentionally deferred work must be recorded in `docs/issues.md` with source file links.

**Panic Hygiene Gate:** Enforce a strict per-phase rule: do not introduce new production `unwrap`/`expect`, and require touched files to be free of production panic calls before closing each phase.

**Compatibility Rule:** No backward compatibility layers or aliases are retained anywhere in this refactor (including crate names, DBus names/paths/interfaces, policy IDs, unit names, and command names).

**Startup Validation Rule:** `storage-service` performs strict preflight validation of naming/config alignment (DBus bus/interface/path, polkit action namespace, and unit/binary references) and fails fast before DBus bind on mismatch.

## Rename Matrix (Validation)

### Crate/Workspace renames
- `storage-ui/` -> `storage-app/`
- `storage-dbus/` -> `storage-udisks/`
- `storage-service-macros/` -> `storage-macros/`
- `storage-common/` -> **removed** (hard cut split into `storage-types/` + `storage-contracts/`)

### Rust package/crate identifier updates
- `storage_dbus` (code/dependency alias) -> `storage_udisks`
- `storage-service-macros` package references -> `storage-macros`
- app crate directory/module references `storage_ui` -> `storage_app` where internal paths require it

### App identity (must stay unchanged)
- package name remains: `cosmic-ext-storage`
- app binary name remains: `cosmic-ext-storage`
- desktop/app-id naming remains tied to `com.cosmic.ext.Storage`

### DBus/public surface rename set (requested)
- Bus name: `org.cosmic.ext.StorageService` -> `org.cosmic.ext.Storage.Service`
- Object paths move to per-concern endpoints (not one shared root object only):
  - `/org/cosmic/ext/Storage/Service/Filesystems`
  - `/org/cosmic/ext/Storage/Service/Image`
  - `/org/cosmic/ext/Storage/Service/Rclone`
  - future concerns follow the same path pattern.
- Interface names:
  - concern-specific interfaces remain separate (no single grouped interface).
  - `org.cosmic.ext.StorageService.Filesystems` -> `org.cosmic.ext.Storage.Service.Filesystems`
  - `org.cosmic.ext.StorageService.Image` -> `org.cosmic.ext.Storage.Service.Image`
  - `org.cosmic.ext.StorageService.Rclone` -> `org.cosmic.ext.Storage.Service.Rclone`

### Policy/config naming updates for DBus rename
- `data/dbus-1/system.d/org.cosmic.ext.StorageService.conf` -> `data/dbus-1/system.d/org.cosmic.ext.Storage.Service.conf`
- `data/polkit-1/actions/org.cosmic.ext.storage-service.policy` -> `data/polkit-1/actions/org.cosmic.ext.storage.service.policy`
- Polkit action IDs namespace:
  - `org.cosmic.ext.storage-service.*` -> `org.cosmic.ext.storage.service.*`
  - keep concern-scoped action IDs (no broad consolidated read/modify-only action model)

### systemd unit/service naming updates
- Rename systemd unit files:
  - `data/systemd/cosmic-storage-service.service` -> `data/systemd/cosmic-ext-storage-service.service`
  - `data/systemd/cosmic-storage-service.socket` -> `data/systemd/cosmic-ext-storage-service.socket`
- Rename service executable:
  - `cosmic-storage-service` -> `cosmic-ext-storage-service` (exact target name)
- Update unit internals (`ExecStart=`, `Also=`, `SyslogIdentifier=`, and related references) to renamed unit/binary identity.

### Internal reference propagation
- Update all docs/specs/comments mentioning old crate names.
- Update all `Cargo.toml` dependency keys/aliases referencing removed/renamed crates.
- Update systemd/dbus/polkit references to the renamed DBus namespace.
- Update scripts/tasks/docs that reference old systemd unit names.

---

### Task 1: Establish workspace standards and enforcement baseline

**Files:**
- Modify: `Cargo.toml`
- Modify: `storage-sys/Cargo.toml`
- Modify: `README.md`
- Modify: `justfile`

**Step 1: Write the failing standards check task**
- Add a `just` target (or equivalent documented command sequence) requiring:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets`
  - `cargo test --workspace --no-run`

**Step 2: Run standards check and verify baseline issues**
Run: `just verify` (or documented equivalent)
Expected: If command missing or inconsistent, FAIL until task is added and documented.

**Step 3: Normalize workspace-level conventions**
- Align edition policy (including `storage-sys`) to the same standard used by the majority of crates.
- Ensure top-level docs describe the canonical verification flow.

**Step 4: Re-run standards check**
Run: `just verify`
Expected: PASS.

#### Task 1 Execution Status (069-polish)

- [x] Added a dedicated `just verify` target with canonical workspace checks.
- [x] Verification command sequence is explicitly codified as `fmt --check`, `clippy --workspace --all-targets`, and `test --workspace --no-run`.
- [x] Aligned `storage-sys` edition with workspace standard (`2024`).
- [x] Documented canonical `just verify` flow in `README.md`.
- [x] Validated Task 1 flow by running `just verify` successfully.

---

### Task 2: Rename `storage-dbus` crate to `storage-udisks`

**Files:**
- Move/Rename: `storage-dbus/` -> `storage-udisks/`
- Modify: `Cargo.toml`
- Modify: `storage-udisks/Cargo.toml`
- Modify: `storage-service/Cargo.toml`
- Modify: any crate `Cargo.toml` referencing old package/lib names
- Modify: docs/spec references to `storage-dbus`

**Step 1: Write failing workspace check for rename state**
Run: `rg -n "storage-dbus|cosmic-ext-storage-dbus" Cargo.toml storage-*/Cargo.toml docs specs`
Expected: Matches found before rename.

**Step 2: Apply crate and package rename**
- Rename directory to `storage-udisks`.
- Rename package/lib identifiers from dbus-specific names to udisks-specific names while keeping non-app crate naming neutral (no `cosmic-ext-` prefix expansion).
- Update workspace members and dependency aliases accordingly.

**Step 3: Re-run rename scan**
Run: `rg -n "storage-dbus|cosmic-ext-storage-dbus" Cargo.toml storage-*/Cargo.toml docs specs`
Expected: No stale references except explicitly documented migration notes (if any).

**Step 4: Compile impacted crates**
Run: `cargo check -p storage-service -p cosmic-ext-storage-udisks -p cosmic-ext-storage`
Expected: PASS.

#### Task 2 Execution Status (069-polish)

- [x] Renamed crate/workspace references from `storage-dbus` to `storage-udisks`.
- [x] Cleared stale `storage-dbus|cosmic-ext-storage-dbus` references in manifests/docs/specs scan.
- [x] Revalidated impacted compile set via equivalent command: `cargo check -p storage-service -p storage-udisks -p cosmic-ext-storage`.

---

### Task 3: Remove stale transitional markers and dead references

**Files:**
- Modify: `storage-app/src/client/mod.rs`
- Modify: `storage-app/src/client/rclone.rs`
- Modify: `storage-udisks/src/encryption/list.rs`
- Modify: any files referencing removed transitional paths discovered during edits

**Step 1: Write failing grep checks for transitional markers**
Run: `rg -n "TODO: Remove when UI integration|TODO: Rewrite to traverse" storage-*/src`
Expected: Matches found (current baseline).

**Step 2: Remove or replace each transitional marker with final code path**
- Delete dead TODO-only pathways.
- Replace temporary traversal logic with the active intended traversal path.

**Step 3: Re-run marker scan**
Run: `rg -n "TODO: Remove when UI integration|TODO: Rewrite to traverse" storage-*/src`
Expected: No matches for those transitional markers.

**Step 4: Compile impacted crates**
Run: `cargo check -p cosmic-ext-storage -p cosmic-ext-storage-udisks`
Expected: PASS.

#### Task 3 Execution Status (069-polish)

- [x] Removed transitional TODO markers from `storage-app/src/client/mod.rs` and `storage-app/src/client/rclone.rs`.
- [x] Removed traversal TODO marker from `storage-udisks/src/encryption/list.rs` by implementing tree traversal listing.
- [x] Removed stale `storage_common` app client import path in `storage-app/src/client/filesystems.rs` after split to `storage-types`/`storage-contracts`.
- [x] Cleared remaining stale source references (including lingering `storage-common` comment/doc mentions) and revalidated with `cargo check -p cosmic-ext-storage -p storage-udisks`.

---

### Task 4: Split `storage-common` into `storage-types` and `storage-contracts`

**Files:**
- Create: `storage-types/Cargo.toml`
- Create: `storage-types/src/lib.rs`
- Create: `storage-contracts/Cargo.toml`
- Create: `storage-contracts/src/lib.rs`
- Modify: `Cargo.toml`
- Delete: `storage-common/`
- Modify: all crate manifests depending on `storage-common`

**Step 1: Write failing dependency graph check**
Run: `cargo metadata --no-deps --format-version 1`
Expected: Graph still includes `storage-common` as the only shared crate before split.

**Step 2: Create split crates and move modules**
- Move DTOs/serde models/enums into `storage-types`.
- Allow lightweight helper methods on DTOs in `storage-types`.
- Move tool traits/contracts into `storage-contracts`.
- Organize `storage-contracts` into `traits` and `protocol` submodules.
- Define multiple narrow traits by concern (discovery, partitioning, filesystem, encryption, usage scan, etc.) instead of one broad toolset trait.
- Model capabilities as optional trait implementations (no forced unsupported stubs for unrelated concerns).
- Ensure `storage-contracts` depends on `storage-types` and uses shared DTOs directly in trait signatures.
- Use `async-trait` for async contract methods.

**Step 3: Rewire all crate dependencies**
- Update `storage-service`, `storage-sys`, `storage-udisks`, `storage-btrfs`, and `storage-app` dependencies.
- Perform a hard-cut migration with no temporary compatibility wrapper crate.

**Step 4: Compile the workspace after split**
Run: `cargo check --workspace`
Expected: PASS.

#### Task 4 Execution Status (069-polish)

- [x] `storage-common` hard-cut split is in place with `storage-types` and `storage-contracts` crates active.
- [x] Workspace/manifests are rewired to `storage-types`/`storage-contracts` dependencies.
- [x] Workspace compile verification for split state has been executed (`cargo check --workspace`).

---

### Task 5: Panic hardening in production code paths

**Files:**
- Modify: `storage-app/src/client/connection.rs`
- Modify: `storage-btrfs/src/subvolume.rs`
- Modify: `storage-types/src/common.rs` (or migrated equivalent)
- Modify: `storage-service/src/filesystems.rs`
- Modify: `storage-service/src/partitions.rs`

**Step 1: Write failing anti-pattern scan**
Run: `rg -n "unwrap\(|expect\(" storage-*/src --glob '!**/tests/**'`
Expected: Matches in production files listed above.

**Step 2: Replace panic points with explicit error handling**
- Convert singleton access, parsing assumptions, and manager construction failures to typed errors.
- Propagate context-rich errors to boundary layers.

**Step 3: Re-run anti-pattern scan for touched files**
Run: `rg -n "unwrap\(|expect\(" storage-app/src/client/connection.rs storage-btrfs/src/subvolume.rs storage-types/src/common.rs storage-service/src/filesystems.rs storage-service/src/partitions.rs`
Expected: No production panic calls remain in touched paths.

**Step 4: Run targeted tests/checks**
Run: `cargo test -p storage-types -p storage-btrfs -p storage-service --no-run`
Expected: PASS.

#### Task 5 Execution Status (069-polish)

- [x] Touched production paths are currently free of `unwrap`/`expect` at the planned locations.
- [x] Targeted verification command passes: `cargo test -p storage-types -p storage-btrfs -p storage-service --no-run`.
- [x] Reviewed wider workspace panic-call scan and kept additional hardening out of this execution stream (remaining matches are primarily test-only assertions plus intentional app initialization expectations).

---

### Task 6: Refactor shared contracts/types for clarity (breaking allowed)

**Files:**
- Modify: `storage-types/src/lib.rs`
- Modify: `storage-types/src/*.rs` (domain DTO modules)
- Modify: `storage-contracts/src/lib.rs`
- Modify: `storage-contracts/src/*.rs` (tool contracts)
- Test: existing module tests in `storage-types/src/*` and `storage-contracts/src/*`

**Step 1: Write failing contract-roundtrip tests for renamed/simplified models**
- Add/adjust serde roundtrip tests for every contract altered.

**Step 2: Run targeted tests to confirm failures before implementation**
Run: `cargo test -p storage-types <new_or_changed_test_name> -v`
Expected: FAIL before model updates.

**Step 3: Implement contract simplification and renames**
- Remove ambiguous fields and duplicate semantic types.
- Use one canonical term per concept across shared DTOs.

**Step 4: Re-run shared contract/type tests**
Run: `cargo test -p storage-types -p storage-contracts -v`
Expected: PASS.

#### Task 6 Execution Status (069-polish)

- [x] Shared contracts/types refactor is functionally active (`storage-types` + `storage-contracts` in use workspace-wide).
- [x] Core protocol/error/operation-id types are centralized in `storage-contracts` with serde tests present.
- [x] Completed per-concern file layout split for `storage-contracts::traits` and `storage-contracts::protocol` via module directories and `mod.rs` re-exports.
- [x] Expanded protocol roundtrip coverage (including `StorageError` serialization) and validated with `cargo test -p storage-contracts -v`.

---

### Task 7: Refactor `storage-sys`/`storage-service` boundary and orchestration flow

**Files:**
- Modify: `storage-sys/src/lib.rs`
- Modify: `storage-sys/src/**`
- Modify: `storage-service/src/main.rs`
- Modify: `storage-service/src/**`

**Step 1: Write failing boundary tests or compile assertions**
- Add or adjust tests that lock desired interface shapes and typed error returns.

**Step 2: Run targeted tests before refactor**
Run: `cargo test -p cosmic-ext-storage-storage-sys -p storage-service --no-run`
Expected: Fail for changed signatures or pending tests.

**Step 3: Implement boundary cleanup**
- Separate policy logic from execution details.
- Keep `storage-service` orchestration-only; external operations must be trait-backed tools.
- Let `storage-service` own concrete adapter wiring/composition for now.
- Use `Arc<dyn Trait + Send + Sync>` trait objects for runtime plugin composition.
- Use fixed routing per concern selected at startup (no per-request dynamic adapter dispatch in this phase).
- Drive fixed startup routing from a small internal config map built via compile-time builder wiring.
- Use a Rust enum for concern keys in the routing map (compile-time exhaustive).
- Fail fast at startup if any required concern lacks a configured adapter.
- Replace ad-hoc wrappers with direct, typed trait interfaces.
- Use a unified `StorageError`/`StorageErrorKind` in `storage-contracts` for tool trait errors.
- Enforce contract boundary: no transport/tool-specific public return types beyond `storage-types`/`storage-contracts`.
- Enforce app/service isolation: no direct service-to-app or app-to-service crate dependency.
- Remove compatibility indirection not needed for alpha.

**Step 4: Re-run checks for both crates**
Run: `cargo check -p cosmic-ext-storage-storage-sys -p storage-service`
Expected: PASS.

#### Task 7 Execution Status (069-polish)

- [x] Switched service concerns to fixed startup routing via `Concern` enum map with required-concern fail-fast checks.
- [x] Migrated service operations to trait-backed adapters for disks, partitions, filesystems, LUKS, and image flows.
- [x] Centralized adapter trait contracts in `storage-contracts/src/traits.rs` and removed duplicate in-crate trait definitions.
- [x] Removed `DiskManager` plumbing from service handlers/routing surface; manager construction is now isolated inside adapter factory wiring.
- [x] Kept runtime composition on `Arc<dyn Trait + Send + Sync>` with startup-time adapter binding.
- [x] Updated service handlers/adapters to import trait contracts directly from `storage-contracts` (no handler dependence on `routing` re-exports).
- [x] Removed `routing` module contract re-export surface; `routing` now consumes contract traits internally only.
- [x] Removed remaining filesystem-handler compatibility indirection (`detect_filesystem_tools` legacy helper path).
- [x] Revalidated Task 7 targeted compile set with `cargo check -p cosmic-ext-storage-storage-sys -p storage-service`.
- [x] Completed remaining Task 7 boundary cleanup items in this execution stream.

---

### Task 8: Normalize `storage-udisks` projection and traversal consistency

**Files:**
- Modify: `storage-udisks/src/lib.rs`
- Modify: `storage-udisks/src/dbus/**`
- Modify: `storage-udisks/src/encryption/**`
- Modify: `storage-udisks/src/**` (transport mapping modules)

**Step 1: Write failing tests for canonical traversal/projection flow**
- Add tests asserting expected node traversal and DBus response projection.

**Step 2: Run UDisks adapter crate tests**
Run: `cargo test -p cosmic-ext-storage-udisks -v`
Expected: FAIL where behavior/shape is intentionally changed.

**Step 3: Implement traversal and naming normalization**
- Remove temporary traversal hacks.
- Align UDisks transport naming and result mapping to updated `storage-types`/`storage-contracts` contracts.
- Keep UDisks-specific structs private to adapter internals.
- Keep UDisks-procedure validation logic in `storage-udisks` (not in `storage-service`).

**Step 4: Re-run DBus checks**
Run: `cargo check -p cosmic-ext-storage-udisks`
Expected: PASS.

#### Task 8 Execution Status (069-polish)

- [x] Replaced LUKS listing traversal TODO stub with canonical volume-tree traversal in `storage-udisks/src/encryption/list.rs`.
- [x] Added traversal-focused unit coverage for LUKS device extraction in `storage-udisks/src/encryption/list.rs`.
- [x] Added traversal/projection coverage for partition flattening order and non-recursive mapping in `storage-udisks/src/disk/discovery.rs`.
- [x] Tightened `storage-udisks` boundary by removing public re-exports of transport-specific zbus/bytestring internals from crate root.
- [x] Revalidated Task 8 checks with `cargo test -p storage-udisks disk::discovery -- --nocapture` and `cargo check -p storage-udisks`.
- [x] Normalized public block-path projection to `String` (`storage_udisks::block_object_path_for_device`) and updated adapter integration.
- [x] Narrowed crate-root exports to device-based operations (removed root exports of object-path transport entry points).
- [x] Completed remaining Task 8 traversal/projection normalization items in this execution stream.

---

### Task 9: Refactor `storage-app` architecture boundaries and state flow

**Files:**
- Modify: `storage-app/src/ui/app/**`
- Modify: `storage-app/src/ui/network/**`
- Modify: `storage-app/src/ui/volumes/**`
- Modify: `storage-app/src/client/**`
- Modify: `storage-app/src/views/**`

**Step 1: Write failing tests for reducer/state transitions in changed flows**
- Focus on settings/network/volumes transition paths touched by module reshaping.

**Step 2: Run targeted UI tests or compile checks**
Run: `cargo test -p cosmic-ext-storage --no-run`
Expected: Baseline compile or targeted tests fail for intentionally changed structure.

**Step 3: Implement UI module cleanup**
- Consolidate duplicated update/view logic.
- Normalize message naming and feature boundaries.
- Remove stale transitional integration code.
- Enforce dependency boundary so `storage-app` links only shared type/contract crates (no service/tool crate deps).
- Complete all crate rename propagation from `storage-ui` to `storage-app` in internal references while keeping user-facing binary/package identity `cosmic-ext-storage`.

**Step 4: Re-run UI checks**
Run: `cargo check -p cosmic-ext-storage`
Expected: PASS.

#### Task 9 Execution Status (069-polish)

- [x] Removed stale transitional integration shim by deleting `storage-app/src/views/dialogs.rs` and dropping `views::dialogs` module export.
- [x] Removed `UiDrive::block_path()` compatibility alias and migrated all app callsites to canonical `UiDrive::device()` API.
- [x] Normalized remaining sidebar/update drive-selection naming from `block_path` wording to `device_path` in app update flow.
- [x] Consolidated duplicated BTRFS selection initialization/update logic in `storage-app/src/ui/volumes/update/selection.rs` into a shared helper.
- [x] Normalized sidebar message boundaries by replacing tuple payload variants with named-field variants (`SidebarSelectDrive { device_path }`, `SidebarDriveEject { device_path }`).
- [x] Revalidated app compile after cleanup with `cargo check -p cosmic-ext-storage`.
- [x] Completed remaining Task 9 UI architecture cleanup slices in this execution stream.

---

### Task 10: Clean and normalize `storage-app/resources`

**Files:**
- Modify/Delete: `storage-app/resources/**`
- Modify: `storage-app/src/**` references to resource paths
- Modify: `storage-app/build.rs` (if packaging/resource indexing requires updates)

**Step 1: Write failing inventory baseline**
Run: `find storage-app/resources -type f | sort`
Expected: Current set includes redundant or non-canonical naming.

**Step 2: Apply resource normalization**
- Keep only packaging-required icon variants.
- Rename provider assets to canonical naming conventions.
- Remove dead/unused resources and update all references.

**Step 3: Verify no broken resource references**
Run: `rg -n "resources/icons|providers/" storage-app/src storage-app/build.rs`
Expected: All paths resolve to current files.

**Step 4: Re-run UI build check**
Run: `cargo check -p cosmic-ext-storage`
Expected: PASS.

#### Task 10 Execution Status (069-polish)

- [x] Captured resource inventory baseline with `find storage-app/resources -type f | sort`.
- [x] Updated stale appstream metadata naming in `storage-app/resources/app.metainfo.xml` (remote icon path and provides ID) to `com.cosmic.ext.Storage`.
- [x] Canonicalized app packaging icon source path in `storage-app/justfile` to use `com.cosmic.ext.Storage.svg` directly.
- [x] Pruned unused duplicate scalable icon asset `storage-app/resources/icons/hicolor/scalable/apps/icon.svg`.
- [x] Re-captured resource inventory post-prune to confirm canonical icon tree.
- [x] Verified provider icon assets are all referenced by active network provider icon mapping (`ui/network/icons.rs` + quick-setup provider set).
- [x] Revalidated app compile after resource metadata updates with `cargo check -p cosmic-ext-storage`.
- [x] Completed remaining Task 10 resource canonicalization items in this execution stream.

---

### Task 11: Align `storage-btrfs` and macro crate with new conventions

**Files:**
- Modify: `storage-btrfs/src/**`
- Modify: `storage-macros/src/lib.rs`

**Step 1: Write failing tests/compile checks for revised interfaces**
- Add focused tests for changed btrfs behavior and macro API shape expectations.

**Step 2: Run targeted checks**
Run: `cargo test -p storage-btrfs -p storage-macros --no-run`
Expected: FAIL before implementation where interfaces are changing.

**Step 3: Implement convention alignment**
- Ensure naming/error patterns match workspace standards.
- Keep proc-macro scope minimal and explicit.

**Step 4: Re-run targeted checks**
Run: `cargo check -p storage-btrfs -p storage-macros`
Expected: PASS.

#### Task 11 Execution Status (069-polish)

- [x] Ran Task 11 targeted baseline check: `cargo test -p storage-btrfs -p storage-macros --no-run`.
- [x] Aligned `storage-macros` DBus/polkit naming examples and default action namespace to current conventions (`Storage.Service` / `storage.service`).
- [x] Aligned `storage-btrfs` crate-level naming/docs language to current project identity (`COSMIC Ext Storage`).
- [x] Trimmed dead internal macro transform state in `storage-macros/src/lib.rs` to keep proc-macro implementation minimal.
- [x] Revalidated targeted compile with `cargo check -p storage-macros -p storage-btrfs`.
- [x] Completed remaining Task 11 convention-alignment slices in this execution stream.

---

### Task 12: Final full-workspace verification and documentation sync

**Files:**
- Modify: `README.md`


**Step 1: Run complete verification suite**
Run: `cargo fmt --all -- --check`
Expected: PASS.

**Step 2: Run lint verification**
Run: `cargo clippy --workspace --all-targets`
Expected: PASS.

**Step 3: Run test compile verification**
Run: `cargo test --workspace --no-run`
Expected: PASS.

**Step 4: Update docs for final architecture terms**
- Update `README.md` only for final architecture/naming terms in this refactor.

**Step 5: Re-run a final quick compile**
Run: `cargo check --workspace`
Expected: PASS.

#### Task 12 Execution Status (069-polish)

- [x] Ran `cargo fmt --all -- --check`.
- [x] Ran `cargo clippy --workspace --all-targets`.
- [x] Ran `cargo test --workspace --no-run`.
- [x] Ran `cargo check --workspace`.
- [x] Completed Task 12 documentation sync step (README architecture/naming audit found no stale rename terms requiring edits).
- [x] Re-ran canonical `just verify` after late-stream cleanup and confirmed warning-free verification output.

---

## PR Slicing Guidance (applies to all tasks)
- Keep **renames/moves** separate from **logic changes** within each PR.
- Keep each PR independently green.
- If a task exceeds one focused review unit, split it before implementation.

## Acceptance Criteria
- Workspace conventions and crate boundaries are consistent.
- Known production panic anti-patterns in targeted paths are removed.
- Resource tree is intentional, canonical, and fully referenced.
- Full workspace validation commands pass.
- No compatibility shims retained unless they demonstrably reduce complexity.
