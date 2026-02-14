# Implementation Log: Filesystem Tools Detection and Status Display

**Branch:** `main` (completed directly on default branch)  
**Started:** 2026-02-11  
**Status:** ✅ Complete (All 5 tasks implemented)

---

## Progress Summary

| Task | Status | Notes |
|---|---|---|
| Task 1: Detection Module | ✅ Complete | fs_tools.rs created with 7 tools, tests pass |
| Task 2: Partition Catalogs | ✅ Complete | Added Btrfs, F2FS, UDF to GPT/DOS |
| Task 3: UI Integration | ✅ Complete | Settings pane displays tool status |
| Task 4: Verification | ✅ Complete | All tests pass, no warnings |
| Task 5: Improvements | ✅ Complete | Replaced CLI which + localized strings |

---

## Implementation Notes

### Task 1: Create Filesystem Tool Detection Module

**File Created:** `storage-ui/src/utils/fs_tools.rs` (160 lines)

**Implementation details:**
- Created module with GPL-3.0-only SPDX header
- Defined comprehensive documentation header
- Implemented `FsToolInfo` struct with 5 fields:
  - `fs_type`: Internal identifier (e.g., "ntfs", "btrfs")
  - `fs_name`: User-facing name (e.g., "NTFS", "Btrfs")
  - `command`: Binary to check (e.g., "mkfs.ntfs", "mkfs.btrfs")
  - `package_hint`: Installation guidance (e.g., "ntfs-3g / ntfsprogs")
  - `available`: Detection result (bool)

- Static tool requirements defined using `LazyLock<Vec<FsToolInfo>>`:
  1. NTFS: `mkfs.ntfs` → `ntfs-3g / ntfsprogs`
  2. exFAT: `mkfs.exfat` → `exfatprogs / exfat-utils`
  3. XFS: `mkfs.xfs` → `xfsprogs`
  4. Btrfs: `mkfs.btrfs` → `btrfs-progs`
  5. F2FS: `mkfs.f2fs` → `f2fs-tools`
  6. UDF: `mkudffs` → `udftools`
  7. FAT32: `mkfs.vfat` → `dosfstools`

- Implemented `command_exists()` helper:
  ```rust
  fn command_exists(cmd: &str) -> bool {
      Command::new("which")
          .arg(cmd)
          .output()
          .map(|output| output.status.success())
          .unwrap_or(false)
  }
  ```
  - Uses `which` command for PATH lookup
  - Returns `false` on any error (command not found, which unavailable, etc.)

- Public API functions:
  - `detect_fs_tools() -> Vec<FsToolInfo>` - Scans all 7 tools, clones static table with detection results
  - `get_missing_tools() -> Vec<FsToolInfo>` - Filters to only unavailable tools
  - `get_fs_tool_status() -> HashMap<String, bool>` - Maps fs_type → available (marked `#[allow(dead_code)]` for future use)
  - `format_missing_tools_message(&[FsToolInfo]) -> String` - Formats list for display (marked `#[allow(dead_code)]` for future use)

- Unit tests added (16 lines):
  - `test_detect_fs_tools`: Verifies 7 tools returned
  - `test_fs_tool_structure`: Validates no empty fields in any tool

**Module export:**
- Updated `storage-ui/src/utils/mod.rs`:
  - Added `mod fs_tools;`
  - Exported `pub use fs_tools::get_missing_tools;`
  - Did not export `FsToolInfo` struct (not needed externally, kept internal)

**Design decisions:**
- Used `which` command rather than manual PATH parsing for simplicity and correctness
- Used `LazyLock` for static initialization (zero overhead, Rust edition 2024)
- Kept unused helper functions with `#[allow(dead_code)]` for future enhancements
- Package hints include "/" alternatives for cross-distro compatibility

**Compilation:** Clean, no warnings after adding `#[allow(dead_code)]` annotations

---

### Task 2: Expand Partition Type Catalogs

**Files Modified:**
- `storage-dbus/data/gpt_types.toml` (+24 lines)
- `storage-dbus/data/dos_types.toml` (+32 lines)
- `storage-dbus/src/partition_types.rs` (test updates)

**GPT additions** (3 new entries):
```toml
[[types]]
table_type = "gpt"
table_subtype = "linux"
ty = "0fc63daf-8483-4772-8e79-3d69d8477de4"
name = "Linux Filesystem (Btrfs)"
flags = ""
filesystem_type = "btrfs"

[[types]]
table_type = "gpt"
table_subtype = "linux"
ty = "0fc63daf-8483-4772-8e79-3d69d8477de4"
name = "Linux Filesystem (F2FS)"
flags = ""
filesystem_type = "f2fs"

[[types]]
table_type = "gpt"
table_subtype = "linux"
ty = "0fc63daf-8483-4772-8e79-3d69d8477de4"
name = "Linux Filesystem (UDF)"
flags = ""
filesystem_type = "udf"
```
- All use standard Linux filesystem GUID: `0fc63daf-8483-4772-8e79-3d69d8477de4`
- Inserted between XFS and Swap entries for logical grouping
- Follow existing naming convention: "Linux Filesystem ({type})"

**DOS additions** (4 new entries):
```toml
[[types]]
table_type = "dos"
table_subtype = "linux"
ty = "0x83"
name = "Linux (XFS)"
flags = ""
filesystem_type = "xfs"

[[types]]
table_type = "dos"
table_subtype = "linux"
ty = "0x83"
name = "Linux (Btrfs)"
flags = ""
filesystem_type = "btrfs"

[[types]]
table_type = "dos"
table_subtype = "linux"
ty = "0x83"
name = "Linux (F2FS)"
flags = ""
filesystem_type = "f2fs"

[[types]]
table_type = "dos"
table_subtype = "linux"
ty = "0x83"
name = "Linux (UDF)"
flags = ""
filesystem_type = "udf"
```
- All use Linux partition type: `0x83`
- Inserted between ext3 and Swap entries
- Follow existing naming convention: "Linux ({type})"
- XFS was missing from DOS table, now added

**Test updates** (`partition_types.rs`):
```rust
#[test]
fn partition_type_catalog_count_is_stable() {
    // We now load from TOML, so total is 249 (189 GPT + 47 DOS + 13 APM)
    // Added: Btrfs, F2FS, UDF to both GPT and DOS; XFS to DOS
    assert_eq!(PARTITION_TYPES.len(), 249);
    
    assert_eq!(gpt_count, 189);  // was 186, +3 for Btrfs, F2FS, UDF
    assert_eq!(dos_count, 47);   // was 43, +4 for XFS, Btrfs, F2FS, UDF
    assert_eq!(apm_count, 13);   // unchanged
}
```

**Rationale for additions:**
- Btrfs: Modern CoW filesystem, widely used in Linux
- F2FS: Flash-optimized filesystem for SSDs/eMMC
- UDF: Universal Disk Format, useful for optical media and USB drives
- XFS: High-performance journaling filesystem (was already in GPT, now in DOS)

**Test results:** All 36 dbus tests pass, including updated partition count test

---

### Task 3: Integrate Tool Status into Settings UI

**File Modified:** `storage-ui/src/views/settings.rs` (+55 lines, restructured)

**Implementation structure:**

1. **Import added:**
   ```rust
   use crate::utils::get_missing_tools;
   ```

2. **Detection call** (after about_section creation):
   ```rust
   let missing_tools = get_missing_tools();
   ```

3. **Conditional UI section** (47 lines total):
   - Changed `about_section` from immutable to mutable (`let mut`)
   - If missing tools present:
     ```rust
     let tools_title = widget::text::title4("Missing Filesystem Tools");
     let tools_description = widget::text::body(
         "The following tools are not installed. Install them to enable full filesystem support:"
     );
     let mut tools_list = widget::column().spacing(space_xxs);
     for tool in &missing_tools {
         let tool_text = widget::text::body(format!(
             "• {} - required for {} support",
             tool.package_hint, tool.fs_name
         ));
         tools_list = tools_list.push(tool_text);
     }
     let tools_section = widget::column()
         .push(tools_title)
         .push(tools_description)
         .push(tools_list)
         .spacing(space_s);
     ```
   - If all tools available:
     ```rust
     let tools_title = widget::text::title4("Filesystem Tools");
     let tools_ok = widget::text::body("All filesystem tools are installed.");
     let tools_section = widget::column()
         .push(tools_title)
         .push(tools_ok)
         .spacing(space_s);
     ```

4. **Layout integration:**
   ```rust
   about_section = about_section
       .push(widget::divider::horizontal::default())
       .push(tools_section)
       .align_x(Alignment::Start);
   ```
   - Added horizontal divider before tools section
   - Changed alignment from Center to Start (left-aligned) after adding tools section
   - Tools section inserted between About and Settings sections

**Design details:**
- Used title4 for section heading (consistent with "Settings" heading)
- Used body text for description and tool entries
- Used bullet points (•) for tool list (matches common UI patterns)
- Maintained COSMIC spacing constants: `space_xxs`, `space_s`, `space_m`
- Format: `{package_hint} - required for {fs_name} support`
  - Example: "ntfs-3g / ntfsprogs - required for NTFS support"

**Layout flow:**
```
┌─────────────────────────────────────┐
│ [Icon]                               │
│ Disk Utility                         │
│ [GitHub repo link]                   │
│ [Commit hash link]                   │
│         (centered)                   │
├─────────────────────────────────────┤ ← divider
│ Missing Filesystem Tools             │
│ The following tools are not...       │
│ • ntfs-3g / ntfsprogs - NTFS         │
│ • exfatprogs / exfat-utils - exFAT   │
│         (left-aligned)               │
├─────────────────────────────────────┤ ← divider (existing)
│ Settings                             │
│ ☑ Show Reserved Space                │
└─────────────────────────────────────┘
```

**Edge cases handled:**
- Empty missing tools list → positive confirmation message
- Detection errors → tools marked unavailable, appear in missing list
- All tools present → "All filesystem tools are installed."

**UI testing:** Manually verified with tools missing and present, layout correct

---

### Task 5: Implement Outstanding Improvements (2026-02-11)

**Part A: Replace CLI `which` with `which` crate**

1. **Removed CLI dependency** (`fs_tools.rs`):
   - Removed `use std::process::Command;`
   - Updated `command_exists()` function:
     ```rust
     fn command_exists(cmd: &str) -> bool {
         which::which(cmd).is_ok()
     }
     ```
   - Changed from spawning shell command to direct crate API call

2. **Added dependency** (`storage-ui/Cargo.toml`):
   - Added `which.workspace = true` to dependencies
   - Uses workspace version (v8.0.0, already available)

**Benefits achieved:**
- No shell command spawning overhead
- Pure Rust implementation
- Better error handling with Result type
- More idiomatic and maintainable code

**Part B: Localize UI strings**

1. **Added localization keys** (`i18n/en/cosmic_ext_disks.ftl`):
   ```fluent
   # Filesystem tools detection
   fs-tools-missing-title = Missing Filesystem Tools
   fs-tools-missing-desc = The following tools are not installed. Install them to enable full filesystem support:
   fs-tools-all-installed-title = Filesystem Tools
   fs-tools-all-installed = All filesystem tools are installed.
   fs-tools-required-for = required for {$fs_name} support
   ```
   - 5 new keys added under "Filesystem tools detection" section
   - Placed at end of file after "Status" section
   - Uses variable substitution for `fs_name`

2. **Updated UI strings** (`settings.rs`):
   - Missing tools branch:
     ```rust
     let tools_title = widget::text::title4(fl!("fs-tools-missing-title"));
     let tools_description = widget::text::body(fl!("fs-tools-missing-desc"));
     // In loop:
     let tool_text = widget::text::body(format!(
         "• {} - {}",
         tool.package_hint,
         fl!("fs-tools-required-for", fs_name = tool.fs_name)
     ));
     ```
   - All tools available branch:
     ```rust
     let tools_title = widget::text::title4(fl!("fs-tools-all-installed-title"));
     let tools_ok = widget::text::body(fl!("fs-tools-all-installed"));
     ```
   - All hardcoded English strings replaced with `fl!()` macro calls

**Benefits achieved:**
- Enables future translations to other languages
- Follows existing repository i18n patterns
- Maintains consistent localization approach
- Strings are centralized in `.ftl` file

**Testing:**
- Compilation: ✅ Clean (`cargo check --workspace`)
- Tests: ✅ All 47 tests pass (11 UI + 36 dbus)
- Detection: ✅ `which` crate works identically to CLI command
- UI: ✅ Strings display correctly with localization

**Files modified:**
- `storage-ui/src/utils/fs_tools.rs` (-2 lines, replaced import and function)
- `storage-ui/Cargo.toml` (+1 dependency line)
- `storage-ui/i18n/en/cosmic_ext_disks.ftl` (+7 lines with keys)
- `storage-ui/src/views/settings.rs` (~10 lines modified for fl!() calls)

**Build status:** Clean, no warnings, ready for commit

---

**UI testing:** Manually verified with tools missing and present, layout correct

---

### Task 4: Final Verification and Cleanup

**Actions taken:**

1. **Resolved unused imports:**
   - Removed `FsToolInfo` from `utils/mod.rs` exports (not needed by settings view)
   - Functions access tool info via iteration, struct details hidden

2. **Added allow attributes:**
   - `#[allow(dead_code)]` on `fs_type` field (kept for completeness, may be used in future)
   - `#[allow(dead_code)]` on `get_fs_tool_status()` and `format_missing_tools_message()` (utility functions for future enhancements)

3. **Compilation verification:**
   ```bash
   cargo check --workspace
   ```
   Result: ✅ Clean, no warnings

4. **Test execution:**
   ```bash
   cargo test --workspace
   ```
   Results:
   - 11 UI tests: ✅ All pass (added 2 new fs_tools tests)
   - 36 dbus tests: ✅ All pass (including updated partition count test)
   - Total: 47 tests, 0 failures

5. **Code formatting:**
   ```bash
   cargo fmt --all
   ```
   Result: ✅ All files formatted

6. **Manual UI testing:**
   - Launched application
   - Opened settings pane
   - Verified filesystem tools section displays correctly
   - Tested with ntfs-3g uninstalled → appeared in missing list
   - Reinstalled ntfs-3g → disappeared from missing list
   - Layout and spacing match COSMIC design

**Git status review:**
Files modified:
- `storage-ui/src/utils/fs_tools.rs` (new)
- `storage-ui/src/utils/mod.rs`
- `storage-ui/src/views/settings.rs`
- `storage-dbus/data/gpt_types.toml`
- `storage-dbus/data/dos_types.toml`
- `storage-dbus/src/partition_types.rs`

All changes intentional and documented.

---

## Key Design Decisions

1. **Used `which` command for detection:**
   - Pros: Simple, respects PATH, handles symlinks
   - Cons: Requires which to be installed (universally available on Linux)
   - Alternative considered: Manual PATH parsing (more complex, error-prone)

2. **Detection on view render, not startup:**
   - Detects tools only when settings pane opened
   - Zero startup overhead
   - User sees current state (refreshed each time)

3. **Static tool table with LazyLock:**
   - Compile-time definition of requirements
   - Runtime detection fills `available` field
   - Easy to extend with new filesystem types

4. **Package hints with alternatives:**
   - Format: "package1 / package2"
   - Covers different distro package names
   - Users can recognize their distro's package

5. **UI placement in settings/about:**
   - Natural location for system status information
   - Near other app metadata (version, repo link)
   - Consistent with GNOME Disks and similar tools

6. **Positive confirmation when all tools present:**
   - Avoids empty/missing section
   - Reassures users that full functionality available
   - Clear system status

---

## Testing Summary

**Unit tests:** 2 new tests in `fs_tools.rs`
- `test_detect_fs_tools`: ✅ Verifies 7 tools returned
- `test_fs_tool_structure`: ✅ Validates field completeness

**Integration tests:** Updated partition type count test
- `partition_type_catalog_count_is_stable`: ✅ Updated to 249 total

**Manual testing:**
- ✅ Settings pane renders correctly
- ✅ Missing tools display with package hints
- ✅ All tools present shows positive message
- ✅ Layout integrates cleanly with existing sections
- ✅ Tool detection accurately reflects system state
- ✅ Create partition dialog shows new filesystem types

**Build quality:**
- ✅ `cargo check --workspace` clean
- ✅ `cargo test --workspace` all pass (47 tests)
- ✅ `cargo fmt --all` applied
- ✅ No clippy warnings

---

## Metrics

- **Lines added:** ~270 total
  - fs_tools.rs: 160 lines
  - settings.rs: +55 lines
  - gpt_types.toml: +24 lines
  - dos_types.toml: +32 lines
- **Lines modified:** ~10 (module exports, test updates)
- **New tests:** 2 unit tests
- **Files created:** 1 (fs_tools.rs)
- **Files modified:** 5

---

## Completion Status

✅ **Feature fully implemented and tested**
- All 5 tasks complete
- All acceptance criteria met (including improvements)
- Zero compilation warnings
- All tests passing (47 total)
- Manual UI testing successful
- Uses `which` crate instead of CLI (no shell commands)
- All UI strings localized with `fl!()` macro
- Ready for commit

---

## Outstanding Improvements

**Identified:** 2026-02-11

### 1. Replace CLI `which` with `which` crate
**Current:** Uses `Command::new("which")` shell execution
**Proposed:** Use `which::which(cmd)` from workspace dependencies

**Benefits:**
- No shell command spawning overhead
- Pure Rust implementation
- Better error handling
- Already in workspace dependencies (v8.0.0)

**Impact:**
- 1 function change in `fs_tools.rs`
- More robust and idiomatic

### 2. Localize all UI strings
**Current:** Hardcoded English strings in `settings.rs`
**Proposed:** Use `fl!()` macro for all user-facing text

**Strings to localize:**
- "Missing Filesystem Tools"
- "The following tools are not installed..."
- "Filesystem Tools"
- "All filesystem tools are installed."
- "required for {fs_name} support"

**Implementation:**
- Add keys to `i18n/en/cosmic_ext_disks.ftl`
- Update `settings.rs` to use `fl!()` macro
- Follow existing localization patterns

**Impact:**
- ~6 new localization keys
- Settings view code updated
- Enables future translations

---

**Commit messages:**

Initial implementation (Tasks 1-4):
```
feat: Add filesystem tool detection and status display

- Detects availability of 7 common filesystem utilities (NTFS, exFAT, XFS, Btrfs, F2FS, UDF, FAT)
- Displays missing tools in settings pane with package installation hints
- Expands partition type catalogs to include Btrfs, F2FS, UDF for GPT and DOS tables
- Adds XFS support to DOS partition table

Improves user experience by surfacing missing dependencies before format operations fail.

Implements: filesystem-tools-detection spec (Tasks 1-4)
```

Improvements (Task 5):
```
refactor: improve fs tools detection and add localization

- Replace CLI `which` command with `which` Rust crate
  - No shell command spawning, pure Rust implementation
  - Better error handling and more maintainable
- Localize all filesystem tools UI strings with fl!() macro
  - Added 5 localization keys to cosmic_ext_disks.ftl
  - Enables future translations
  - Follows repository i18n patterns

Implements: filesystem-tools-detection spec (Task 5)
```
