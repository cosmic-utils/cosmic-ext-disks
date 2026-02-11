# Plan: Filesystem Tools Detection and Status Display

**Branch:** `main` (completed directly on default branch)  
**Source:** User brief (2026-02-11)  
**Status:** ✅ Implemented

---

## Context

The application supports creating and formatting various filesystem types (NTFS, exFAT, XFS, Btrfs, F2FS, UDF, FAT), but users have no visibility into which filesystem utilities are installed on their system. When attempting to format a partition with a filesystem whose tools are missing, the operation fails with a cryptic error from UDisks2.

This creates issues:
- Users don't discover missing dependencies until they try to use a feature and it fails
- No guidance about which packages need to be installed for specific filesystem support
- Poor discoverability of supported filesystem types vs. actually available types

Other disk utilities (like GNOME Disks) detect tool availability and inform users about missing dependencies upfront, improving the user experience.

---

## Goals

1. **Detect filesystem tool availability** at runtime using PATH lookup
2. **Display missing tools** in the application's about/settings pane with:
   - Clear identification of which tools are missing
   - Package names users should install
   - Which filesystem types each tool enables
3. **Cover common filesystems:**
   - NTFS (`ntfs-3g`)
   - exFAT (`exfatprogs`)
   - XFS (`xfsprogs`)
   - Btrfs (`btrfs-progs`)
   - F2FS (`f2fs-tools`)
   - UDF (`udftools`)
   - FAT32 (`dosfstools`)
4. **Expand partition type catalog** to include the additional filesystem types (Btrfs, F2FS, UDF) for both GPT and DOS partition tables

---

## Non-Goals

- Preventing users from attempting to format with unavailable tools (UDisks2 already handles error reporting)
- Automatic installation of missing tools (suggest packages only)
- Tool version detection or compatibility checking
- Runtime re-detection (check once at startup)
- Support for non-standard tool locations outside PATH
- Detection of read-only vs. read-write tool availability (assume utilities provide both)

---

## Proposed Approach

### A) Tool Detection Module (disks-ui/utils)

Create a new utility module `disks-ui/src/utils/fs_tools.rs`:

1. **Define tool requirements:**
   - Static table mapping filesystem types to:
     - Filesystem type identifier (e.g., "ntfs", "btrfs")
     - Human-readable filesystem name (e.g., "NTFS", "Btrfs")
     - Command to check (`mkfs.ntfs`, `mkfs.btrfs`)
     - Package hint for installation (`ntfs-3g`, `btrfs-progs`)

2. **Implement detection logic:**
   - Use `which` command to check PATH availability
   - Execute during module initialization using LazyLock
   - Return list of `FsToolInfo` structs with availability status

3. **Public API:**
   ```rust
   pub struct FsToolInfo {
       pub fs_type: &'static str,
       pub fs_name: &'static str,
       pub command: &'static str,
       pub package_hint: &'static str,
       pub available: bool,
   }
   
   pub fn detect_fs_tools() -> Vec<FsToolInfo>
   pub fn get_missing_tools() -> Vec<FsToolInfo>
   pub fn get_fs_tool_status() -> HashMap<String, bool>
   pub fn format_missing_tools_message(&[FsToolInfo]) -> String
   ```

4. **Edge cases:**
   - `which` command not available → tool marked unavailable
   - Empty PATH → all marked unavailable
   - No missing tools → display positive confirmation message

### B) Settings UI Integration (disks-ui/views/settings.rs)

Enhance the settings/about pane to display filesystem tools status:

1. **Section structure:**
   - Add new section after "About" titled "Filesystem Tools" or "Missing Filesystem Tools"
   - If tools are missing:
     - Show title: "Missing Filesystem Tools"
     - Description: "The following tools are not installed..."
     - Bulleted list of missing tools with format:
       `• package-name - required for FILESYSTEM support`
   - If all tools present:
     - Show title: "Filesystem Tools"
     - Message: "All filesystem tools are installed."
   - Use `widget::divider::horizontal::default()` to separate sections

2. **UI implementation:**
   - Call `get_missing_tools()` on each settings view render
   - Build column widget dynamically based on results
   - Use `text::title4()` for section heading
   - Use `text::body()` for description and tool entries
   - Maintain existing COSMIC theme spacing (space_xxs, space_s)

3. **Layout:**
   ```
   [About Section - icon, title, repo link, git info]
   ─────────────────────────────
   [Filesystem Tools Section]
   Missing Filesystem Tools
   The following tools are not installed...
   • ntfs-3g - required for NTFS support
   • exfatprogs - required for exFAT support
   ─────────────────────────────
   [Settings Section - toggles...]
   ```

### C) Partition Type Catalog Expansion (disks-dbus/data)

Add missing filesystem types to TOML partition catalogs:

1. **GPT types** (`disks-dbus/data/gpt_types.toml`):
   - Add Btrfs entry (table_subtype = "linux", filesystem_type = "btrfs")
   - Add F2FS entry (table_subtype = "linux", filesystem_type = "f2fs")
   - Add UDF entry (table_subtype = "linux", filesystem_type = "udf")
   - All use standard Linux filesystem GUID: `0fc63daf-8483-4772-8e79-3d69d8477de4`

2. **DOS types** (`disks-dbus/data/dos_types.toml`):
   - Add XFS entry (ty = "0x83", filesystem_type = "xfs")
   - Add Btrfs entry (ty = "0x83", filesystem_type = "btrfs")
   - Add F2FS entry (ty = "0x83", filesystem_type = "f2fs")
   - Add UDF entry (ty = "0x83", filesystem_type = "udf")
   - All use Linux partition type 0x83

3. **Update tests:**
   - Adjust partition count assertions in `partition_types.rs`
   - New totals: 249 total (189 GPT + 47 DOS + 13 APM)

---

## User/System Flows

### Flow 1: User Opens Settings (Tools Missing)
1. User navigates to settings pane (hamburger menu → About/Settings)
2. Settings view renders, calls `get_missing_tools()`
3. Detection finds `ntfs-3g` and `exfatprogs` are missing
4. UI displays:
   ```
   Missing Filesystem Tools
   The following tools are not installed. Install them to enable full filesystem support:
   • ntfs-3g / ntfsprogs - required for NTFS support
   • exfatprogs / exfat-utils - required for exFAT support
   ```
5. User installs packages using system package manager
6. User restarts application (or tools become available immediately if PATH updated)

### Flow 2: User Opens Settings (All Tools Available)
1. User navigates to settings
2. Detection finds all tools present
3. UI displays:
   ```
   Filesystem Tools
   All filesystem tools are installed.
   ```
4. User sees confirmation that full functionality is available

### Flow 3: User Attempts to Format with Missing Tool
1. User selects partition → Format
2. User chooses "NTFS" from filesystem dropdown
3. User confirms format dialog
4. Format operation calls UDisks2 with filesystem_type = "ntfs"
5. UDisks2 fails because `mkfs.ntfs` not found
6. Application shows error dialog with UDisks2 error message
7. User navigates to settings, sees NTFS is listed as missing
8. User installs `ntfs-3g`, restarts, successfully formats

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| `which` command not available on some systems | Detection fails, all tools marked unavailable | Document assumption; alternative: check `/usr/bin/` and `/usr/local/bin/` directly |
| Tool available but broken/non-functional | False positive (shown as available) | Acceptable; UDisks2 will still report actual format errors |
| Package names vary across distros | User confusion about package names | Use common names with `/` alternatives (e.g., "ntfs-3g / ntfsprogs") |
| Detection adds startup latency | Slower app launch | LazyLock ensures check only happens when settings view opened |
| New filesystem types added without tools | Incomplete detection coverage | Document maintenance process for adding new types |

---

## Acceptance Criteria

- [x] Detection module checks for 7 common filesystem tools (NTFS, exFAT, XFS, Btrfs, F2FS, UDF, FAT)
- [x] Settings pane displays missing tools with package hints
- [x] Settings pane shows positive message when all tools available
- [x] GPT partition catalog includes Btrfs, F2FS, UDF entries
- [x] DOS partition catalog includes XFS, Btrfs, F2FS, UDF entries
- [x] Partition type tests updated to reflect new counts
- [x] All tests pass (`cargo test --workspace`)
- [x] Code compiles without warnings (`cargo check --workspace`)
- [x] UI integrates cleanly with existing settings layout
- [x] Tool detection handles edge cases (missing commands, unavailable tools)
