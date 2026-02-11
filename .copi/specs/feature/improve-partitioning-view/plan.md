# Plan: Improve Partitioning View UI/UX

**Branch:** `feature/improve-partitioning-view`  
**Source:** User brief (2026-02-11)  
**Applies to:** Create Partition dialog, Format Partition dialog

---

## Context

The Create Partition and Format Partition dialogs currently have several UX issues:

1. **Name field always visible**: The "Volume Name" field is shown even for DOS/MBR partition tables, which don't support partition names (only filesystem labels).

2. **Size control limitations**: 
   - Users must use byte-based values without unit selection
   - Input updates happen immediately on keystroke, making precise entry difficult
   - No ability to switch between KB, MB, GB, TB for easier input

3. **Unclear erase semantics**: 
   - Uses a toggle/switch control ("Erase")
   - Label doesn't convey that this is a slow, destructive overwrite operation
   - Inconsistent with the Format Disk dialog which uses descriptive labels like "Overwrite (Slow)"

4. **Filesystem type discoverability**:
   - Uses a compact dropdown that hides available options
   - No indication which filesystem types require additional tools
   - Users discover missing dependencies only when operations fail
   - No tooltip guidance about filesystem type choices

5. **LUKS label ambiguity**:
   - Checkbox says "Password Protected" without indicating the underlying technology (LUKS)
   - Users may not understand this creates a LUKS-encrypted volume

Recent work added `FSTools` detection (showing missing tools in settings), but this information isn't surfaced in the partitioning dialogs where users make filesystem type decisions.

---

## Goals

1. **Conditional UI for partition names**
   - Hide "Volume Name" field when partition table type is DOS/MBR
   - Show "Volume Name" field for GPT (and other table types that support partition names)
   - Always show filesystem label (which is set in `format_options`)

2. **Improved size input control**
   - Create reusable component with per-field unit selection (B, KB, MB, GB, TB)
   - Defer updates until focus loss, Enter key, or explicit finish editing
   - Replace existing byte-only labelled_spinner controls
   - Make reusable across other dialogs that need size input

3. **Clearer erase control**
   - Change from `toggler` (switch) to `checkbox`
   - Update label from "erase" to "Overwrite Data (Slow)"
   - Align with Format Disk dialog terminology

4. **Radio list for filesystem types**
   - Replace dropdown with expanded radio button list
   - Show all common partition types at once (better discoverability)
   - Integrate FSTools detection: grey out/disable types requiring missing tools
   - Add tooltips showing tool requirements (e.g., "ntfs-3g / ntfsprogs required")
   - Display format: `{filesystem_name} — {short_description}`
   - No partition type IDs/UUIDs shown to users
   - **All strings must be localized via `fl!()` macro** (filesystem names and descriptions)
   
   **GPT options (11 types):**
   1. ext4 — Modern Linux filesystem (default)
   2. ext3 — Legacy Linux filesystem
   3. XFS — High-performance journaling
   4. Btrfs — Copy-on-write with snapshots
   5. F2FS — Flash-optimized filesystem
   6. UDF — Universal Disk Format
   7. NTFS — Windows filesystem
   8. FAT32 — Universal compatibility
   9. exFAT — Large files, cross-platform
   10. Swap — Virtual memory
   
   **DOS/MBR options (10 types):**
   1. ext4 — Modern Linux filesystem (default)
   2. ext3 — Legacy Linux filesystem
   3. XFS — High-performance journaling
   4. Btrfs — Copy-on-write with snapshots
   5. F2FS — Flash-optimized filesystem
   6. UDF — Universal Disk Format
   7. NTFS — Windows filesystem
   8. FAT32 — Universal compatibility
   9. exFAT — Large files, cross-platform
   10. Swap — Virtual memory

5. **Clearer LUKS labeling**
   - Update checkbox label from "Password Protected" to "Password Protected (LUKS)"
   - Keep always visible (LUKS is universally supported via UDisks2)

6. **Apply improvements to both dialogs**
   - Create Partition dialog
   - Format Partition dialog

---

## Non-Goals

- **Not** hiding LUKS checkbox based on tool availability (cryptsetup is assumed present; UDisks2 handles LUKS operations)
- **Not** creating a multi-page wizard (keep single-page dialog with improved organization)
- **Not** changing backend logic for partition creation/formatting
- **Not** modifying the Edit Partition dialog (different use case)
- **Not** adding filesystem type descriptions beyond tool requirements (keep tooltips concise)
- **Not** implementing automatic tool installation (detection/guidance only)

---

## Proposed Approach

### A) Conditional Partition Name Field

**Logic:**
- Check `CreatePartitionInfo.table_type` 
- If `table_type == "dos"`, omit the partition name text_input
- For all other types (especially GPT), show the partition name field

**Implementation:**
- In `disks-ui/src/ui/dialogs/view/partition.rs`:
  - `create_partition()`: conditionally push name field based on `create.table_type`
  - `format_partition()`: same conditional logic
- Backend already handles this correctly in `disks-dbus/src/disks/ops.rs`:
  ```rust
  let create_name = if table_type == "dos" {
      ""
  } else {
      info.name.as_str()
  };
  ```

**Testing:**
- Create partition on GPT disk → name field visible, name used
- Create partition on DOS/MBR disk → name field hidden, empty name passed to backend

---

### B) Unit-Aware Size Input Component

**Component requirements:**
- Text input field with unit dropdown (B, KB, MB, GB, TB)
- Label above input
- Deferred update: only emit message on blur/Enter/finish editing
- Display current size in selected unit
- Convert to/from bytes for backend

**API design:**
```rust
pub fn unit_size_input<'a, Message>(
    label: String,
    value_bytes: u64,
    on_change: impl Fn(u64) -> Message + 'a,
) -> Element<'a, Message>
```

**Location:**
- New module: `disks-ui/src/utils/unit_size_input.rs`
- Export from `disks-ui/src/utils/mod.rs`

**Usage in dialogs:**
- Replace existing `labelled_spinner` calls for "Partition Size" and "Free Space"
- Pass `CreateMessage::SizeUpdate` as callback
- Component maintains internal state for pending edits

**Unit conversion:**
- 1 KB = 1024 bytes (binary units, consistent with `bytes_to_pretty`)
- Validate input range against min/max_size
- Show validation errors inline

**Additional considerations:**
- Keep slider for visual feedback (remove labelled_spinner, add unit_size_input alongside slider)
- Slider updates immediately; unit inputs defer updates
- When slider moves, update unit_size_input display but don't trigger callbacks until user edits text

---

### C) Erase Control: Toggler → Checkbox

**Changes:**
- `disks-ui/src/ui/dialogs/view/partition.rs`:
  - `create_partition()`: replace `toggler(create_clone.erase)` with `checkbox(fl!("overwrite-data-slow"), create_clone.erase)`
  - `format_partition()`: same change

**i18n:**
- Add to `disks-ui/i18n/en/cosmic_ext_disks.ftl`:
  ```ftl
  overwrite-data-slow = Overwrite Data (Slow)
  ```
- Swedish translation (`sv/`):
  ```ftl
  overwrite-data-slow = Skriv över data (långsamt)
  ```
- Note: Filesystem type names and descriptions also require i18n (see Task 1)

**Consistency:**
- Aligns with Format Disk dialog's "Overwrite (Slow)" option

---

### D) Filesystem Type Radio List with FSTools Integration

**Current state:**
- `dropdown(valid_partition_types, Some(index), |v| ...)` 
- `valid_partition_types` is `Vec<String>` from `get_valid_partition_names(table_type)`

**New design:**
1. **Get partition type info + tool availability:**
   ```rust
   let partition_types = match create.table_type.as_str() {
       "gpt" => &disks_dbus::COMMON_GPT_TYPES,
       "dos" => &disks_dbus::COMMON_DOS_TYPES,
       _ => &[],
   };
   
   let fs_tool_status = crate::utils::fs_tools::get_fs_tool_status();
   // Returns HashMap<String, bool> mapping fs_type → available
   ```

2. **Build radio list:**
   - Use `cosmic::widget::radio` for each partition type
   - Grey out if required tool is missing
   - Add tooltip with tool requirement info

3. **Radio widget structure:**
   ```rust
   // Helper to generate friendly display name from partition type
   fn friendly_filesystem_label(p_type: &PartitionTypeInfo) -> String {
       // Map filesystem_type to user-friendly descriptions (all localized)
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
   }
   
   for (idx, p_type) in partition_types.iter().enumerate() {
       let selected = idx == create.selected_partition_type_index;
       let tool_available = fs_tool_status
           .get(p_type.filesystem_type.as_str())
           .copied()
           .unwrap_or(true); // Default to available for ext4, swap, etc.
       
       let mut radio = widget::radio(
           friendly_filesystem_label(p_type),
           idx,
           create.selected_partition_type_index,
           |v| CreateMessage::PartitionTypeUpdate(v).into()
       );
       
       if !tool_available {
           radio = radio.style(theme::Radio::Disabled);
           // Or use a custom style that shows greyed-out but selectable for "try anyway"
       }
       
       // Add tooltip
       if !tool_available {
           radio = widget::tooltip(
               radio,
               fl!("fs-tools-required-for", fs_name = p_type.filesystem_type),
               widget::tooltip::Position::Top
           );
       }
       
       content = content.push(radio);
   }
   ```

4. **Filesystem type mapping:**
   - `PartitionTypeInfo.filesystem_type` field contains: "ext4", "ntfs", "vfat", "xfs", "btrfs", "f2fs", "udf", "exfat", "swap"
   - `FSTools` detects: "ntfs", "exfat", "xfs", "btrfs", "f2fs", "udf", "vfat"
   - Always-available types (no tools needed): "ext4", "swap"

5. **Tooltip text reuse:**
   - Use same i18n keys as settings page: `fs-tools-required-for`
   - Include package hints from `FsToolInfo.package_hint`

**Visual layout:**
- Replace single dropdown row with vertical stack of radio buttons
- Group visually with a caption heading: `caption_heading(fl!("filesystem-type"))`

---

### E) Update LUKS Checkbox Label

**Simple change:**
- In both `create_partition()` and `format_partition()`:
  ```rust
  checkbox(fl!("password-protected-luks"), create.password_protected)
      .on_toggle(|v| CreateMessage::PasswordProtectedUpdate(v).into())
  ```

**i18n:**
- Update `disks-ui/i18n/en/cosmic_ext_disks.ftl`:
  ```ftl
  password-protected-luks = Password Protected (LUKS)
  ```
- Swedish:
  ```ftl
  password-protected-luks = Lösenordsskyddad (LUKS)
  ```
- Note: All filesystem names/descriptions must also use `fl!()` (see Task 1)

**Always visible:** 
- No conditional logic; LUKS is always supported via UDisks2 and cryptsetup is a standard system component

---

### F) Dialog Layout Organization

**Create Partition dialog order:**
1. Partition Name (conditional: only if table_type != "dos")
2. Size controls:
   - Slider (immediate visual feedback)
   - Unit-aware "Partition Size" input
   - Unit-aware "Free Space" input
3. Overwrite Data (Slow) - checkbox
4. Filesystem Type - radio list with tool status
5. Password Protected (LUKS) - checkbox
6. Password fields (conditional: only if password_protected == true)
7. Error caption (if any)
8. Action buttons (Cancel, Continue)

**Format Partition dialog differences:**
- Omit size controls (fixed size = selected volume size)
- Otherwise same structure

---

## User/System Flows

### Flow 1: Create Partition on GPT Disk (ext4, no encryption)
1. User right-clicks free space → "Create Partition"
2. Dialog opens:
   - "Volume Name" field visible (GPT supports names)
   - Size slider + two unit-aware inputs (default: size in MB)
   - "Overwrite Data (Slow)" checkbox unchecked
   - Radio list shows ext4, NTFS (greyed if missing), exFAT (greyed if missing), etc.
   - ext4 selected by default
   - "Password Protected (LUKS)" unchecked
3. User types name, adjusts size, leaves other defaults
4. User clicks "Continue"
5. Backend creates partition with name, label, size, no encryption

### Flow 2: Create DOS/MBR Partition with NTFS (tool missing)
1. User right-clicks free space on DOS/MBR disk → "Create Partition"
2. Dialog opens:
   - "Volume Name" field **hidden** (DOS doesn't support partition names)
   - Size controls visible
   - NTFS option shown but greyed out
   - Hovering over NTFS shows tooltip: "ntfs-3g / ntfsprogs - required for NTFS support"
3. User tries to select NTFS → still selectable (backend will fail gracefully with hint)
4. Or user sees grey indicator and switches to FAT32
5. Continue → partition created

### Flow 3: Format Existing Partition with Encryption
1. User selects partition → Volume actions → "Format Partition"
2. Dialog opens:
   - Name field visible (if GPT) or hidden (if DOS/MBR)
   - **No size controls** (size fixed to selected volume)
   - Radio list of filesystem types with tool status
   - "Password Protected (LUKS)" checkbox visible
3. User checks "Password Protected (LUKS)"
4. Password fields appear below
5. User enters passphrase, confirms
6. Continue → backend formats with LUKS + selected filesystem type inside

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Unit-aware input increases complexity, potential for conversion bugs | Medium | Medium | Extensive testing with boundary values; reuse `bytes_to_pretty` conversion logic; unit tests for conversions |
| Radio list with many options clutters dialog | Low | Low | Common types list is already curated (~7-10 options); vertical space is acceptable for better discoverability |
| Greyed-out options confuse users ("why can't I select this?") | Medium | Low | Clear tooltips with actionable package names; users can still select greyed options (soft warning vs hard block) |
| Conditional name field causes confusion (field appears/disappears between disks) | Low | Low | Behavior is table-type dependent (DOS vs GPT); most modern systems use GPT; clear from context |
| Deferred size updates feel laggy | Low | Low | Standard behavior for numeric inputs in desktop apps; blur/Enter is expected UX |

---

## Acceptance Criteria

**UI behavior:**
- [ ] Partition name field hidden when `table_type == "dos"`; shown for GPT
- [ ] Unit-aware size inputs allow selecting B/KB/MB/GB/TB, show converted value
- [ ] Size inputs only update state on blur/Enter (not per-keystroke)
- [ ] "Erase" control replaced with checkbox labeled "Overwrite Data (Slow)"
- [ ] Filesystem types displayed as radio button list (not dropdown)
- [ ] Filesystem types requiring missing tools are greyed out
- [ ] Hovering over greyed filesystem shows tooltip with tool/package info
- [ ] LUKS checkbox labeled "Password Protected (LUKS)"
- [ ] Both Create Partition and Format Partition dialogs updated

**Functional correctness:**
- [ ] DOS/MBR partition creation passes empty name to backend (existing behavior maintained)
- [ ] GPT partition creation passes user-entered name to backend
- [ ] Size conversions accurate (bytes ↔ KB/MB/GB/TB)
- [ ] FSTools detection correctly maps to partition types (ntfs → requires ntfs-3g, etc.)
- [ ] Tooltips display same text as settings page
- [ ] Selecting greyed-out filesystem type still works (soft warning, not hard error)
- [ ] Backend behavior unchanged (validation/errors same as before)

**i18n coverage:**
- [ ] New strings added to `en/cosmic_ext_disks.ftl` (22 total: 2 labels + 10 fs-name + 10 fs-desc)
- [ ] Swedish translations added to `sv/cosmic_ext_disks.ftl` (22 total)
- [ ] All UI-visible strings use `fl!()` macro (no hardcoded English)

**Code quality:**
- [ ] Unit-aware input component reusable (clean API, no dialog-specific coupling)
- [ ] No regressions in existing dialog functionality
- [ ] clippy and rustfmt pass
- [ ] Manual testing on GPT and DOS/MBR disks

---

## Open Questions

1. **Should greyed-out filesystem types be completely unselectable (hard block) or greyed but still selectable (soft warning)?**
   - **Decision needed**: Soft warning preferred (user can still try; backend error provides specific hint)

2. **Should the unit-aware input preserve user's last-selected unit across dialog re-openings?**
   - **Decision needed**: Not critical for MVP; default to MB (most human-friendly for partition sizes)

3. **Should Format Partition dialog also show partition name field?**
   - **Decision needed**: Yes, for consistency; format operations can update filesystem label

4. **Should we show a warning icon or color for missing tools, or just grey + tooltip?**
   - **Decision needed**: Grey + tooltip sufficient; visual clutter vs. clarity tradeoff

---

## Implementation Notes

- `disks-ui/src/ui/dialogs/view/partition.rs` will see the most changes
- `disks-ui/src/utils/unit_size_input.rs` is new (reusable component)
- `disks-ui/i18n/en/cosmic_ext_disks.ftl` and `sv/` need new strings
- FSTools integration via `disks-ui/src/utils/fs_tools.rs` (already implemented)
- No changes to `disks-dbus` crate (backend logic unchanged)

---

## References

- [FSTools Detection Spec](./../feature/filesystem-tools-detection/plan.md)
- [Format Disk Dialog Spec](./../feature/format-disk-dialog/plan.md)
- [Create Partition Password Protection Fix](./../fix/create-partition-password-protection/plan.md)
