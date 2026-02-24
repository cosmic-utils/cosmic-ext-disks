# 069 Polish Design

## Scope
This design defines a single “polish” PR that combines UX refinement and service architecture cleanup while preserving existing D-Bus contracts and user workflows.

## Goals
- Add branded rclone provider icons with compliant licensing and deterministic fallback behavior.
- Keep the sidebar `Images` section header always visible.
- Move non-trivial input flows from dialogs to the shared wizard shell system.
- Clean and simplify the Settings pane layout and repository affordances.
- Refactor `storage-service` into orchestration-focused handlers with trait-driven domain modules.
- Gate filesystem-tool-dependent behavior behind compile-time feature flags across workspace crates, default-enabled.

## Architecture Boundaries
- D-Bus handlers are thin orchestrators: auth/context validation, trait delegation, error mapping.
- Domain logic is moved behind per-domain traits and concrete implementations:
  - filesystems
  - disks
  - partitions
  - luks
  - lvm
  - image
  - rclone
  - btrfs
- Existing D-Bus method names and payload contracts remain stable.
- Capability/tool availability becomes explicit service state consumed by UI.

## UI/UX Design
### Branded Rclone Icons
- Add provider icon resolution chain:
  1. License-compliant icon crate/provider set
  2. Bundled local SVG fallback (same license constraints)
  3. Generic symbolic icon fallback
- Apply consistently in:
  - Network sidebar rows
  - Network wizard provider tile grid

### Sidebar Images Section
- Render `Images` section header unconditionally in sidebar tree.
- Keep image actions visually tied to the `Images` section and right-aligned.

### Wizard Migration Policy
- Migrate dialogs that represent multi-field or procedural workflows to shared wizard primitives (`wizard_shell`, `wizard_action_row`, tile helpers).
- Keep simple single-value prompts and confirmation dialogs as lightweight dialogs.

### Settings Pane Cleanup
- Simplify About content density.
- Move commit hash/date into caption-sized text.
- Place GitHub icon button in the bottom-right corner.
- Place the commit caption immediately to the left of the GitHub icon.

## Data Flow
### Service
Request → auth/context checks → trait call → feature/capability gate check → typed domain result → D-Bus response + tracing.

### UI
User action → app message/update → wizard/dialog state transition → client request → capability-aware response handling → sidebar/nav/detail refresh.

## Error Handling
- Feature-disabled paths return deterministic "unsupported/unavailable" responses, not ambiguous failures.
- User-facing errors stay concise and actionable.
- Internal logs retain detailed operation context (`device`, `path`, `operation`).

## Validation Strategy
- Preserve API-level compatibility for existing clients.
- Validate key flows: mount/unmount/format/image/network and wizard transitions.
- Run workspace quality gates: `cargo fmt`, `cargo clippy`, and relevant tests.
- Add targeted tests for trait adapters and feature-flag compile combinations where feasible.

## Non-Goals
- No net-new UX flows beyond requested polish and migration.
- No D-Bus contract redesign.
- No expansion of simple confirmation/value-only dialogs into wizards.
