# Tasks: Improve Partitioning View UI/UX

**Branch:** `feature/improve-partitioning-view`

This document breaks down the implementation into commit-sized tasks. Each task should be independently testable.

---

## Task 1: Add i18n strings for new labels

**Scope**: Add localization strings for new UI labels.

**Files:**
- `disks-ui/i18n/en/cosmic_ext_disks.ftl`
- `disks-ui/i18n/sv/cosmic_ext_disks.ftl`

**Steps:**
1. Add to `en/cosmic_ext_disks.ftl`:
   ```ftl
   overwrite-data-slow = Overwrite Data (Slow)
   password-protected-luks = Password Protected (LUKS)
   
   # Filesystem type names
   fs-name-ext4 = ext4
   fs-name-ext3 = ext3
   fs-name-xfs = XFS
   fs-name-btrfs = Btrfs
   fs-name-f2fs = F2FS
   fs-name-udf = UDF
   fs-name-ntfs = NTFS
   fs-name-vfat = FAT32
   fs-name-exfat = exFAT
   fs-name-swap = Swap
   
   # Filesystem type descriptions
   fs-desc-ext4 = Modern Linux filesystem (default)
   fs-desc-ext3 = Legacy Linux filesystem
   fs-desc-xfs = High-performance journaling
   fs-desc-btrfs = Copy-on-write with snapshots
   fs-desc-f2fs = Flash-optimized filesystem
   fs-desc-udf = Universal Disk Format
   fs-desc-ntfs = Windows filesystem
   fs-desc-vfat = Universal compatibility
   fs-desc-exfat = Large files, cross-platform
   fs-desc-swap = Virtual memory
   ```
2. Add to `sv/cosmic_ext_disks.ftl`:
   ```ftl
   overwrite-data-slow = Skriv över data (långsamt)
   password-protected-luks = Lösenordsskyddad (LUKS)
   
   # Filsystemtypnamn
   fs-name-ext4 = ext4
   fs-name-ext3 = ext3
   fs-name-xfs = XFS
   fs-name-btrfs = Btrfs
   fs-name-f2fs = F2FS
   fs-name-udf = UDF
   fs-name-ntfs = NTFS
   fs-name-vfat = FAT32
   fs-name-exfat = exFAT
   fs-name-swap = Växlingsutrymme
   
   # Beskrivningar av filsystemtyper
   fs-desc-ext4 = Modernt Linux-filsystem (standard)
   fs-desc-ext3 = Äldre Linux-filsystem
   fs-desc-xfs = Högpresterande journalföring
   fs-desc-btrfs = Copy-on-write med ögonblicksbilder
   fs-desc-f2fs = Flash-optimerat filsystem
   fs-desc-udf = Universal Disk Format
   fs-desc-ntfs = Windows-filsystem
   fs-desc-vfat = Universell kompatibilitet
   fs-desc-exfat = Stora filer, multiplattform
   fs-desc-swap = Virtuellt minne
   ```
3. Verify no duplicate keys exist
4. Run `cargo build` to ensure i18n compilation succeeds

**Test plan:**
- Build succeeds
- `fl!("overwrite-data-slow")` and `fl!("password-protected-luks")` resolve correctly at runtime
- All filesystem name strings (`fs-name-*`) resolve correctly
- All filesystem description strings (`fs-desc-*`) resolve correctly
- Swedish translations present for all new keys

**Done when:**
- [ ] All UI strings added to both language files (en + sv)
- [ ] 10 filesystem name keys (fs-name-*)
- [ ] 10 filesystem description keys (fs-desc-*)
- [ ] 2 label keys (overwrite-data-slow, password-protected-luks)
- [ ] No build errors
- [ ] All strings resolve correctly via `fl!` macro at runtime

---

## Task 2: Change Erase control from toggler to checkbox

**Scope**: Replace switch/toggler with checkbox in both dialogs, apply new label.

**Files:**
- `disks-ui/src/ui/dialogs/view/partition.rs`

**Steps:**
1. In `create_partition()` function:
   - Locate: `toggler(create_clone.erase).label(fl!("erase")).on_toggle(...)`
   - Replace with: `checkbox(fl!("overwrite-data-slow"), create_clone.erase).on_toggle(...)`

2. In `format_partition()` function:
   - Locate: `toggler(create.erase).label(fl!("erase")).on_toggle(...)`
   - Replace with: `checkbox(fl!("overwrite-data-slow"), create.erase).on_toggle(...)`

3. Verify imports: `use cosmic::widget::{button, checkbox, dialog, dropdown, slider, text_input};`
   - `checkbox` should already be imported; if not, add it

4. Build and run:
   - `cargo build --package cosmic-ext-disks`
   - Launch app, open Create Partition dialog
   - Verify checkbox renders instead of toggle
   - Verify label reads "Overwrite Data (Slow)"
   - Test checking/unchecking updates state correctly

**Test plan:**
- Open Create Partition dialog → checkbox visible with new label
- Open Format Partition dialog → checkbox visible with new label
- Check/uncheck updates internal state (`info.erase` toggles)
- Backend behavior unchanged (checked = erase:true, unchecked = erase:false)

**Done when:**
- [ ] Both dialogs use checkbox widget
- [ ] Label uses `fl!("overwrite-data-slow")`
- [ ] Functional behavior identical to previous toggler
- [ ] No visual/layout regressions

---

## Task 3: Update LUKS checkbox label

**Scope**: Change "Password Protected" to "Password Protected (LUKS)" in both dialogs.

**Files:**
- `disks-ui/src/ui/dialogs/view/partition.rs`

**Steps:**
1. In `create_partition()` function:
   - Locate: `checkbox(fl!("password-protected"), create.password_protected)`
   - Replace label with: `checkbox(fl!("password-protected-luks"), create.password_protected)`

2. Verify same pattern in `format_partition()` if applicable:
   - Check if format_partition dialog already has password protection checkbox
   - If yes, update label similarly
   - If no, skip (current code inspection shows format_partition doesn't have password protection; this may be intentional)

3. Build and verify:
   - Launch app, open Create Partition dialog
   - Verify checkbox label reads "Password Protected (LUKS)"

**Test plan:**
- Open Create Partition dialog → checkbox label shows "(LUKS)"
- Check checkbox → password fields appear
- Uncheck checkbox → password fields disappear
- Functional behavior unchanged

**Done when:**
- [ ] LUKS checkbox label updated in Create Partition dialog
- [ ] Label uses `fl!("password-protected-luks")`
- [ ] No functional changes
- [ ] Password field appearance logic still works

---

## Task 4: Conditional partition name field visibility

**Scope**: Hide "Volume Name" field when table type is DOS/MBR; show for other types.

**Files:**
- `disks-ui/src/ui/dialogs/view/partition.rs`

**Steps:**
1. In `create_partition()` function:
   - Locate where `text_input` for volume name is added to `content`
   - Wrap in conditional:
     ```rust
     if create.table_type != "dos" {
         content = content.push(
             text_input(fl!("volume-name"), create_clone.name)
                 .label(fl!("volume-name"))
                 .on_input(|t| CreateMessage::NameUpdate(t).into())
         );
     }
     ```

2. In `format_partition()` function:
   - Same conditional logic:
     ```rust
     if create.table_type != "dos" {
         content = content.push(
             text_input(fl!("volume-name"), create.name.clone())
                 .label(fl!("volume-name"))
                 .on_input(|t| CreateMessage::NameUpdate(t).into())
         );
     }
     ```

3. Test with GPT disk:
   - Create free space on GPT disk
   - Right-click → Create Partition
   - Verify name field is visible

4. Test with DOS/MBR disk:
   - Create free space on DOS/MBR disk
   - Right-click → Create Partition
   - Verify name field is **hidden**

5. Backend validation:
   - Verify `disks-dbus/src/disks/ops.rs::build_create_partition_and_format_args` still handles DOS correctly:
     ```rust
     let create_name = if table_type == "dos" {
         ""
     } else {
         info.name.as_str()
     };
     ```
   - This logic should already be present (no changes needed)

**Test plan:**
- GPT: name field visible, can enter name, name passed to backend
- DOS/MBR: name field hidden, empty name passed to backend
- Format Partition: same behavior
- No backend errors or validation issues

**Done when:**
- [ ] Name field hidden for DOS/MBR table types
- [ ] Name field visible for GPT and other types
- [ ] Backend correctly receives empty name for DOS, user name for GPT
- [ ] Works in both Create and Format dialogs

---

## Task 5: Create unit-aware size input component (reusable utility)

**Scope**: Build reusable size input widget with unit selection and deferred updates.

**Files:**
- `disks-ui/src/utils/unit_size_input.rs` (new file)
- `disks-ui/src/utils/mod.rs` (export new component)

**Steps:**
1. Create `disks-ui/src/utils/unit_size_input.rs`

2. Define component API:
   ```rust
   use cosmic::{Element, iced_widget, widget::{text_input, dropdown}};
   
   pub enum SizeUnit {
       Bytes,
       Kilobytes,
       Megabytes,
       Gigabytes,
       Terabytes,
   }
   
   impl SizeUnit {
       pub fn to_bytes(&self, value: f64) -> u64 {
           match self {
               SizeUnit::Bytes => value as u64,
               SizeUnit::Kilobytes => (value * 1024.0) as u64,
               SizeUnit::Megabytes => (value * 1024.0 * 1024.0) as u64,
               SizeUnit::Gigabytes => (value * 1024.0 * 1024.0 * 1024.0) as u64,
               SizeUnit::Terabytes => (value * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
           }
       }
       
       pub fn from_bytes(&self, bytes: u64) -> f64 {
           match self {
               SizeUnit::Bytes => bytes as f64,
               SizeUnit::Kilobytes => bytes as f64 / 1024.0,
               SizeUnit::Megabytes => bytes as f64 / (1024.0 * 1024.0),
               SizeUnit::Gigabytes => bytes as f64 / (1024.0 * 1024.0 * 1024.0),
               SizeUnit::Terabytes => bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0),
           }
       }
       
       pub fn label(&self) -> &'static str {
           match self {
               SizeUnit::Bytes => "B",
               SizeUnit::Kilobytes => "KB",
               SizeUnit::Megabytes => "MB",
               SizeUnit::Gigabytes => "GB",
               SizeUnit::Terabytes => "TB",
           }
       }
       
       pub fn all() -> Vec<String> {
           vec!["B".into(), "KB".into(), "MB".into(), "GB".into(), "TB".into()]
       }
       
       pub fn from_index(idx: usize) -> Self {
           match idx {
               0 => SizeUnit::Bytes,
               1 => SizeUnit::Kilobytes,
               2 => SizeUnit::Megabytes,
               3 => SizeUnit::Gigabytes,
               4 => SizeUnit::Terabytes,
               _ => SizeUnit::Megabytes, // default
           }
       }
   }
   
   // State management would need to be external (in dialog state)
   // For now, provide a helper to render the input + unit selector
   pub fn unit_size_input<'a, Message: Clone + 'a>(
       label: String,
       value_bytes: u64,
       unit_index: usize,
       on_size_change: impl Fn(u64) -> Message + 'a,
       on_unit_change: impl Fn(usize) -> Message + 'a,
   ) -> Element<'a, Message> {
       let unit = SizeUnit::from_index(unit_index);
       let display_value = unit.from_bytes(value_bytes);
       let display_string = format!("{:.2}", display_value);
       
       let input = text_input(&display_string, &display_string)
           .label(&label)
           .on_submit(/* parse and call on_size_change */)
           .on_blur(/* same */);
       
       let unit_selector = dropdown(
           SizeUnit::all(),
           Some(unit_index),
           on_unit_change
       );
       
       iced_widget::row![input, unit_selector]
           .spacing(8)
           .into()
   }
   ```

3. Add module export in `disks-ui/src/utils/mod.rs`:
   ```rust
   pub mod unit_size_input;
   pub use unit_size_input::{unit_size_input, SizeUnit};
   ```

4. Write unit tests for conversions:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn megabytes_to_bytes() {
           assert_eq!(SizeUnit::Megabytes.to_bytes(100.0), 104857600);
       }
       
       #[test]
       fn bytes_to_gigabytes() {
           let gb = SizeUnit::Gigabytes.from_bytes(1073741824);
           assert!((gb - 1.0).abs() < 0.001);
       }
   }
   ```

5. Build and test:
   - `cargo test --package cosmic-ext-disks unit_size_input`
   - Verify conversions are accurate

**Test plan:**
- Unit tests pass for all conversion combinations
- No rounding errors for common sizes (1 GB, 500 MB, etc.)

**Done when:**
- [ ] `unit_size_input.rs` created with conversion logic
- [ ] Unit tests pass
- [ ] Module exported from `utils/mod.rs`
- [ ] No compilation errors

**Note:** This task creates the utility but doesn't integrate it into dialogs yet. Integration happens in Task 6.

---

## Task 6: Integrate unit-aware size inputs in partition dialogs

**Scope**: Replace labelled_spinner controls with unit-aware inputs; add state for tracking units.

**Files:**
- `disks-ui/src/ui/dialogs/state.rs`
- `disks-ui/src/ui/dialogs/view/partition.rs`
- `disks-ui/src/ui/dialogs/message.rs`
- `disks-ui/src/ui/volumes/update/create.rs`

**Steps:**

1. **Add state for unit tracking** in `state.rs`:
   ```rust
   #[derive(Debug, Clone)]
   pub struct CreatePartitionDialog {
       pub info: CreatePartitionInfo,
       pub running: bool,
       pub error: Option<String>,
       pub size_unit_index: usize, // 0=B, 1=KB, 2=MB, 3=GB, 4=TB
   }
   ```
   Default `size_unit_index: 2` (MB) when constructing

2. **Add messages** in `message.rs`:
   ```rust
   pub enum CreateMessage {
       // ...existing variants...
       SizeUnitUpdate(usize),
       // or keep SizeUpdate(u64) and let component manage unit internally
   }
   ```

3. **Update dialog view** in `view/partition.rs`:
   - Import: `use crate::utils::{unit_size_input, SizeUnit};`
   - Replace `labelled_spinner` calls:
     ```rust
     // Remove:
     // labelled_spinner(fl!("partition-size"), size_pretty, ...)
     
     // Add:
     content = content.push(
         unit_size_input(
             fl!("partition-size"),
             create.size,
             state.size_unit_index,
             |v| CreateMessage::SizeUpdate(v).into(),
             |v| CreateMessage::SizeUnitUpdate(v).into(),
         )
     );
     ```
   - Keep slider for visual feedback:
     ```rust
     slider(0.0..=len, size, |v| CreateMessage::SizeUpdate(v as u64).into())
     ```

4. **Handle unit change messages** in `update/create.rs`:
   ```rust
   CreateMessage::SizeUnitUpdate(unit_idx) => {
       state.size_unit_index = unit_idx;
       // No need to recalculate size; display updates automatically
   }
   ```

5. **Test with slider + input coordination**:
   - Move slider → input display updates to new value
   - Type in input, press Enter → size updates, slider moves
   - Change unit → display value changes, underlying bytes unchanged
   - Blur input field → updates propagate

6. **Repeat for Format Partition dialog** (no size controls, so skip)

**Test plan:**
- Slider and unit input stay in sync
- Changing units doesn't alter underlying byte value
- Typing precise values updates state correctly
- Deferred updates work (no updates on every keystroke)

**Done when:**
- [ ] Unit-aware inputs replace labelled_spinner in Create Partition
- [ ] State tracks selected unit
- [ ] User can type values and switch units freely
- [ ] No visual or functional regressions

**Note:** This task may require iteration if text_input.on_blur doesn't exist in cosmic::widget. Alternative: use on_submit + manual focus tracking.

---

## Task 7: Replace filesystem type dropdown with radio list

**Scope**: Expand dropdown to radio button list, show all options at once.

**Files:**
- `disks-ui/src/ui/dialogs/view/partition.rs`

**Steps:**

1. In `create_partition()`:
   - Locate: `dropdown(valid_partition_types, Some(...), |v| ...)`
   - Replace with radio button loop:
     ```rust
     // Get partition type details
     use disks_dbus::{COMMON_GPT_TYPES, COMMON_DOS_TYPES};
     let partition_types: &[disks_dbus::PartitionTypeInfo] = match create.table_type.as_str() {
         "gpt" => &COMMON_GPT_TYPES,
         "dos" => &COMMON_DOS_TYPES,
         _ => &[],
     };
     
     // Helper to generate friendly display labels (all localized)
     let friendly_label = |p_type: &disks_dbus::PartitionTypeInfo| -> String {
         let description = match p_type.filesystem_type.as_str() {
             "ext4" => fl!("fs-desc-ext4"),
             "ext3" => fl!("fs-desc-ext3"),
             "xfs" => fl!("fs-desc-xfs"),
             "btrfs" => fl!("fs-desc-btrfs"),
             "f2fs" => fl!("fs-desc-f2fs"),
             "udf" => fl!("fs-desc-udf"),
             "ntfs" => fl!("fs-desc-ntfs"),
             "vfat" => fl!("fs-desc-vfat"),
             "exfat" => fl!("fs-desc-exfat"),
             "swap" => fl!("fs-desc-swap"),
             _ => String::new(),
         };
         
         let fs_name = match p_type.filesystem_type.as_str() {
             "vfat" => fl!("fs-name-vfat"),
             "xfs" => fl!("fs-name-xfs"),
             "btrfs" => fl!("fs-name-btrfs"),
             "f2fs" => fl!("fs-name-f2fs"),
             "udf" => fl!("fs-name-udf"),
             "ntfs" => fl!("fs-name-ntfs"),
             "exfat" => fl!("fs-name-exfat"),
             "swap" => fl!("fs-name-swap"),
             "ext4" => fl!("fs-name-ext4"),
             "ext3" => fl!("fs-name-ext3"),
             fs => fs.to_string(),
         };
         
         if description.is_empty() {
             fs_name
         } else {
             format!("{} — {}", fs_name, description)
         }
     };
     
     // Add caption heading
     content = content.push(caption_heading(fl!("filesystem-type")));
     
     // Radio buttons with friendly labels (no UUIDs/type IDs shown)
     for (idx, p_type) in partition_types.iter().enumerate() {
         let selected = idx == create.selected_partition_type_index;
         let label = friendly_label(p_type);
         
         let radio = widget::radio(
             label,
             idx,
             Some(create.selected_partition_type_index),
             |v| CreateMessage::PartitionTypeUpdate(v).into()
         );
         
         content = content.push(radio);
     }
     ```

2. Verify imports:
   ```rust
   use cosmic::widget::{button, checkbox, dialog, radio, slider, text_input};
   use disks_dbus::{COMMON_GPT_TYPES, COMMON_DOS_TYPES};
   ```

3. Test rendering:
   - Open Create Partition dialog
   - Verify radio list renders vertically
   - Verify selecting different types updates state
   - Ensure previously selected type is pre-selected

4. Repeat in `format_partition()` function

**Test plan:**
- Radio list visible with all common partition types (10-11 options depending on table type)
- Labels show friendly names without UUIDs/type IDs (e.g., "ext4 — Modern Linux filesystem (default)")
- Selecting radio button updates `selected_partition_type_index`
- Selection persists when dialog re-renders
- Backend receives correct partition type index
- Order: Linux filesystems → Windows filesystems → Swap

**Done when:**
- [x] Dropdown replaced with radio list in Create Partition
- [x] Dropdown replaced with radio list in Format Partition
- [x] All partition types visible at once
- [x] Selection works correctly

---

## Task 8: Integrate FSTools detection and grey out unavailable types

**Scope**: Detect missing filesystem tools, grey out corresponding radio buttons, add tooltips.

**Files:**
- `disks-ui/src/ui/dialogs/view/partition.rs`

**Steps:**

1. Get FSTools status:
   ```rust
   use crate::utils::fs_tools;
   let tool_status = fs_tools::get_fs_tool_status();
   // Returns HashMap<String, bool>
   ```

2. For each radio button, check tool availability:
   ```rust
   for (idx, p_type) in partition_types.iter().enumerate() {
       let label = friendly_label(p_type); // Uses helper from Task 7
       
       // Check if this filesystem type requires tools
       let tool_available = tool_status
           .get(p_type.filesystem_type.as_str())
           .copied()
           .unwrap_or(true); // Default true for ext4, swap (no special tools)
       
       let mut radio = widget::radio(
           label.clone(),
           idx,
           Some(create.selected_partition_type_index),
           |v| CreateMessage::PartitionTypeUpdate(v).into()
       );
       
       // Grey out if tool is missing
       if !tool_available {
           // Use a custom style or cosmic::theme::Radio variant for disabled appearance
           // Cosmic radio may not have a built-in "greyed but enabled" style,
           // so we may need to wrap in a container with opacity or use text color
           radio = radio.style(theme::Radio::Disabled); // or custom style
           
           // Add tooltip
           let tooltip_text = fl!(
               "fs-tools-required-for",
               fs_name = p_type.filesystem_type.as_str()
           );
           radio = widget::tooltip(
               radio,
               tooltip_text,
               widget::tooltip::Position::Top
           );
       }
       
       content = content.push(radio);
   }
   ```

3. Verify tooltip imports:
   ```rust
   use cosmic::widget::tooltip;
   ```

4. Test with missing tools:
   - Uninstall ntfs-3g: `sudo apt remove ntfs-3g` (or equivalent)
   - Launch app, open Create Partition dialog
   - Verify NTFS radio button is greyed and shows tooltip on hover
   - Reinstall ntfs-3g, verify NTFS is no longer greyed

5. Test tooltip text:
   - Verify tooltip says "ntfs-3g / ntfsprogs - required for NTFS support"
   - Match format from settings page

**Test plan:**
- Missing tools → corresponding filesystem types greyed
- Hover shows tooltip with package name and filesystem name
- Tooltip text matches settings page format
- User can still select greyed options (soft warning, not hard block)

**Done when:**
- [x] FSTools status retrieved in dialog view
- [x] Radio buttons greyed when tools missing
- [x] Tooltips show package hints + filesystem names
- [x] Text matches settings page i18n strings
- [x] Works for both Create and Format dialogs

---

## Task 9: Manual testing and edge cases

**Scope**: Comprehensive testing across different scenarios.

**Test scenarios:**

1. **GPT disk with all tools installed:**
   - Name field visible
   - All filesystem types selectable, none greyed
   - Size inputs work with all units
   - LUKS encryption works end-to-end

2. **DOS/MBR disk with missing NTFS tools:**
   - Name field hidden
   - NTFS greyed with tooltip
   - FAT32 available
   - Size controls work correctly

3. **Format existing partition (GPT):**
   - Name field visible
   - No size controls shown
   - Filesystem type radio list works
   - LUKS option available

4. **Switch between units rapidly:**
   - Change size unit from MB → GB → KB
   - Verify display updates correctly
   - Verify underlying byte value stable
   - Type new value in KB, press Enter → updates correctly

5. **Validation edge cases:**
   - Enter size larger than available space → verify error
   - Enter negative size → verify error or input rejection
   - Select NTFS without tools → verify backend error is clear

6. **i18n:**
   - Switch to Swedish locale (if possible)
   - Verify translated strings appear correctly
   - Verify tooltip text is localized

7. **Accessibility:**
   - Tab navigation works through radio list
   - Keyboard selection works (Space/Enter on focused radio)
   - Screen reader can read labels and tooltips (manual test if available)

**Done when:**
- [ ] All test scenarios pass
- [ ] No regressions in existing functionality
- [ ] Error messages clear and actionable
- [ ] UI feels responsive and intuitive

---

## Task 10: Update documentation and close spec

**Scope**: Document changes in commit messages and mark spec as complete.

**Steps:**

1. Review all commits for:
   - Clear, descriptive commit messages
   - Conventional Commits format (e.g., `feat(ui): add unit-aware size inputs`)
   - Link to spec in commit body if relevant

2. Update spec status:
   - Edit `.copi/specs/feature/improve-partitioning-view/plan.md`
   - Change status to "Implemented"
   - Add completion date

3. Create summary commit message:
   ```
   feat(ui): improve partitioning view UX
   
   - Hide partition name field for DOS/MBR tables
   - Add unit-aware size inputs (B/KB/MB/GB/TB) with deferred updates
   - Replace Erase toggle with "Overwrite Data (Slow)" checkbox
   - Display filesystem types as radio list instead of dropdown
   - Grey out filesystem types with missing tools, show tooltips
   - Update LUKS label to "Password Protected (LUKS)"
   - Apply improvements to Create Partition and Format Partition dialogs
   
   Closes: feature/improve-partitioning-view
   ```

4. Run final checks:
   - `cargo clippy --workspace --all-features` → no warnings
   - `cargo fmt --all --check` → formatted
   - `cargo test --workspace` → all tests pass
   - `cargo build --release` → successful

**Done when:**
- [ ] All quality gates pass
- [ ] Commit messages follow conventions
- [ ] Spec marked complete
- [ ] Ready for PR and manual testing

---

## Dependency Summary

- **Task 1** (i18n): no dependencies
- **Task 2** (checkbox): depends on Task 1
- **Task 3** (LUKS label): depends on Task 1
- **Task 4** (conditional name): independent
- **Task 5** (unit input component): independent
- **Task 6** (integrate unit inputs): depends on Task 5
- **Task 7** (radio list): independent of other UI tasks
- **Task 8** (FSTools integration): depends on Task 7
- **Task 9** (testing): depends on Tasks 2-8 complete
- **Task 10** (documentation): depends on all tasks

**Recommended order:** 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10

**Parallelizable:** Tasks 1, 4, 5, 7 can be done in any order. Tasks 2 and 3 both depend on Task 1 but are otherwise independent.
