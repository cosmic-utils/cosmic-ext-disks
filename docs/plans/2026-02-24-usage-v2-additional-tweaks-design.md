# Usage V2 Additional Tweaks Design

## Summary
Apply a second round of Usage UX refinements across bar/chip layout, filter behavior, icons, refresh/configure flow, string localization, and category-system planning.

This design keeps rclone wizard layout as the canonical baseline for wizard visuals and keeps advanced distro-aware category expansion in design/spec mode for now.

## Goals
- Double usage bar height for better visual weight.
- Switch category controls from single tab selection to multi-filter chips.
- Order category chips by descending category size.
- Add relevant category icons (chips + file list leading column).
- Rename `Number of files` to `Files per Category`.
- Split actions so `Refresh` re-runs current configuration and `Configure` reopens wizard.
- Sweep UI-facing strings into i18n keys.
- Define a robust distro-aware category expansion architecture for follow-up implementation.

## Non-Goals
- No implementation of full distro-aware category engine in this cycle.
- No new pages or modal systems.
- No policy enforcement in UI for privileged categories.

## Architecture

### Usage bar and filters
- Increase usage segmented bar height from current 18px to 36px.
- Replace single `selected_category` tab behavior with a selected-set filter model:
  - default: all visible categories selected,
  - toggle chip on/off to include/exclude,
  - prevent empty selection by retaining at least one selected category.
- Chip ordering is computed each render from scan result category totals sorted descending.

### Service/UI responsibility split for restricted categories
- **Service is authoritative** for privileged/restricted category exposure.
- In non-root mode (or denied/limited ACL scope), service returns no data for restricted categories.
- UI does not gate categories by mode; it simply renders non-zero categories from returned data.

### Refresh/configure behavior
- `Refresh` triggers a new scan immediately with current confirmed configuration.
- Add `Configure` button adjacent to `Refresh` to open wizard and edit scan settings.
- Wizard remains configuration entrypoint; refresh does not reopen wizard.

### i18n strategy
- Replace hardcoded usage/wizard/action/category strings with `fl!` keys.
- Add category label keys, action labels/tooltips, and new root-mode wording.
- Include icon accessibility labels where surfaced.

## UX Specification

### 1) Usage bar
- Height doubled to 36px.
- Placement unchanged (top of usage results section).

### 2) Category controls
- Chips appear directly below usage bar.
- Chips are multi-select filters, not mutually exclusive tabs.
- Chip order = descending category bytes.
- Each chip includes a relevant icon + localized label + formatted byte total.
- Default selection includes all currently visible categories.

### 3) File list
- Add leading colored icon column mapping each row to its category.
- Columns become: category icon, filename, size.
- Existing selection interactions remain unchanged.

### 4) Action bar
- Rename `Number of files` to `Files per Category`.
- Add `Configure` button next to `Refresh`.
- `Refresh` reruns scan with current persisted configuration.

### 5) Root-mode wording
- Rename toggle label to `Show All Files (Root Mode)`.

## Category Expansion (Design-Only)

### Proposed category set
- Documents
- Images
- Audio
- Video
- Archives
- Code
- Binaries
- Packages
- System
- Cache
- Logs
- Containers/VMs
- Backups
- Other

### Distro-aware architecture
- `DistroDetector` from `/etc/os-release`.
- `PackageOwnershipProvider` abstraction with distro adapters (`dpkg`, `rpm`, `pacman`, etc.).
- Rule pipeline priority:
  1. explicit system path rules,
  2. package ownership resolution,
  3. cache/log/path semantic rules,
  4. container/vm path rules,
  5. extension/type fallback,
  6. `Other`.
- Missing adapter/tooling falls back gracefully to non-package rules.

### Extension opportunities
- Flatpak/Snap/AppImage awareness.
- Language/package cache families (`cargo`, `npm`, `pip`, etc.).
- User-defined rule overlays in future config.

## Data Flow
1. User adjusts configuration in wizard (`Configure`) and starts scan.
2. Configuration persists in usage state for subsequent `Refresh` operations.
3. Service scans with current config and returns category totals/top files.
4. Service omits/reduces restricted categories when privilege scope does not allow them.
5. UI sorts non-zero categories by size descending and renders filter chips.
6. Selected chip set filters displayed files across included categories.

## Error Handling
- Preserve existing wizard and scan error surfaces.
- If service returns no categories after filtering/privilege constraints, show existing empty-state pattern.
- Keep delete and selection behavior unchanged except file source set now reflects active multi-filter categories.

## Testing Strategy

### Reducer/state tests
- Refresh uses current config and does not open wizard.
- Configure opens wizard.
- Multi-filter selection defaults to all visible categories and enforces non-empty selection.

### View/layout checks
- Usage bar height update is applied.
- Category chips sort descending by bytes.
- Chip/icon rendering and leading file-row icon column present.

### Service/UI contract checks
- Non-root scans can return zero restricted-category data.
- UI only renders non-zero categories (no mode-based hard gate).

### i18n checks
- No new user-facing hardcoded strings in touched Usage/Wizard UI.

## Acceptance Criteria
- Usage bar is twice previous height.
- Category controls are descending-size multi-filter chips with icons.
- File list includes colored category icon first column.
- Action label is `Files per Category`.
- `Refresh` reruns with current config; `Configure` opens wizard.
- Root-mode label reads `Show All Files (Root Mode)`.
- Service controls restricted-category visibility; UI shows non-zero categories only.
- Usage-facing strings introduced/changed by this work are localized.