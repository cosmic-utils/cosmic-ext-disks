# Workspace Cleanup and Refactor Design

## Summary
Perform a deep, phased refactor across the entire workspace to improve structure, naming consistency, error handling, and maintainability.

This project explicitly assumes **no backward compatibility requirement** because the product is alpha with no current users.

## Constraints and Assumptions
- Breaking internal APIs is allowed.
- We optimize for clarity and long-term architecture over compatibility adapters.
- Delivery is via **phased PRs** to control risk and review scope.
- Every phase must keep workspace health green (`fmt`, `clippy`, test compile).

## Current-State Analysis

### Crates in scope
- `storage-common`
- `storage-sys`
- `storage-dbus`
- `storage-service`
- `storage-service-macros`
- `storage-btrfs`
- `storage-ui`

### Resources audit
- Runtime resources currently live under `storage-ui/resources`.
- Cleanup focus:
  - canonical naming for provider and app icons,
  - remove dead/duplicate assets,
  - ensure one authoritative icon per role/size unless required by platform packaging.

### Structural and quality signals
- Mixed edition baseline (`storage-sys` still on 2021 while other crates are 2024).
- Transitional TODOs remain in integration paths (UI client and DBus traversal areas).
- Panic-prone production paths exist (`unwrap`/`expect` in non-test code), especially around:
  - service manager construction,
  - UI connection singleton access,
  - utility parsing assumptions.
- Workspace currently compiles and lints cleanly, enabling non-emergency architectural work.

## Refactor Objectives
1. Establish coherent crate boundaries and naming conventions.
2. Remove transitional/legacy code and compatibility layers.
3. Replace panic-prone production flows with explicit error propagation.
4. Normalize resource organization and asset naming.
5. Standardize module structure patterns across crates.
6. Preserve behavior where sensible, but allow intentional breaking redesigns.

## Non-Goals
- No end-user migration tooling.
- No deprecation shim period.
- No UI feature expansion beyond cleanup/refactor necessities.

## Architecture Direction

### Boundary model
- `storage-common`: stable internal domain contracts and shared DTOs.
- `storage-sys`: system/probe/scan primitives with explicit, typed errors.
- `storage-dbus`: DBus-facing projection and transport mapping.
- `storage-service`: orchestration and policy layer.
- `storage-ui`: presentation/state orchestration with thin client adapters.
- `storage-btrfs`: filesystem-specific operations isolated from service policy.
- `storage-service-macros`: authorization/proc-macro concerns only.

### Naming and module conventions
- Prefer domain-oriented module names over operation-history names.
- Align message/event naming patterns across UI and service call paths.
- Use one canonical term per concept (e.g., avoid mixed “scan/job/task” terms unless semantically different).

### Error handling policy
- No `unwrap`/`expect` in production paths.
- Convert latent panics to typed errors with context.
- Bubble errors to boundary layers where user/log-facing decisions are made.

## Phased PR Plan

### PR1: Workspace normalization and guardrails
- Unify edition/manifest conventions where feasible.
- Standardize workspace lint/test/fmt expectations.
- Document naming/error-handling rules for all crates.
- Remove low-risk dead code and stale TODOs.

**Exit criteria**
- Shared conventions documented and applied.
- Workspace checks green.

### PR2: `storage-common` contract cleanup
- Consolidate shared models and rename ambiguous types/fields.
- Remove legacy shapes not needed for alpha compatibility.
- Tighten serde behavior and contract clarity.

**Exit criteria**
- Internal consumers compile against simplified contracts.
- Serialization tests reflect new canonical models.

### PR3: `storage-sys` + `storage-service` boundary refactor
- Replace panic-prone construction/initialization with explicit errors.
- Clarify service orchestration interfaces and split policy from execution.
- Remove accidental complexity and temporary wrappers.

**Exit criteria**
- Clear service/system boundary with typed failure paths.
- No production unwrap/expect in touched modules.

### PR4: `storage-dbus` transport consistency pass
- Normalize traversal/data projection from domain to DBus.
- Remove temporary traversal hacks/TODO-path logic.
- Align method/result naming with common contracts.

**Exit criteria**
- DBus mapping code has consistent flow and naming.
- Service-DBus integration compiles cleanly after contract updates.

### PR5: `storage-ui` architecture and resources cleanup
- Simplify app/update/view module boundaries and client adapters.
- Remove stale transitional integration code.
- Normalize resources under `storage-ui/resources`:
  - canonical icon names,
  - remove unused assets,
  - keep packaging-required variants only.

**Exit criteria**
- UI module layout is coherent and predictable.
- Resources tree is minimal and intentional.

### PR6: `storage-btrfs` + macros polish
- Align Btrfs crate naming/error patterns with new workspace standards.
- Ensure proc-macro crate API stays minimal and purpose-specific.

**Exit criteria**
- Crate responsibilities are narrow and explicit.
- Cross-crate integration remains green.

## Data and Control Flow Refinement
- Normalize request/response and event naming end-to-end:
  - UI message → client call → service action → DBus projection.
- Prefer explicit operation structs for multi-parameter workflows.
- Remove duplicated transformation logic; keep mapping at one boundary layer.

## Validation Strategy

### Per-PR validation
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace --no-run`

### Focused validation
- Add/adjust unit tests where refactors alter contract behavior.
- Prefer targeted module tests for renamed/restructured flows.

## Risks and Mitigations
- **Risk:** deep refactor drift across crates.
  - **Mitigation:** strict phased boundaries and per-PR exit criteria.
- **Risk:** accidental behavior regression while removing temporary layers.
  - **Mitigation:** preserve integration checks and incremental compile gates.
- **Risk:** broad rename churn reducing review quality.
  - **Mitigation:** isolate pure rename commits from logic changes in each PR.

## Deliverables
- A sequence of phased PRs implementing the six waves above.
- Updated docs reflecting final module boundaries and naming conventions.
- Cleaned resource tree under `storage-ui/resources`.
- Removal of identified production panic anti-patterns in refactored paths.

## Implementation Handoff
After design approval, generate an execution plan that translates each PR wave into concrete tasks (file-level work items, verification commands, and acceptance checks) before touching implementation code.
