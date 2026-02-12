app-title = Disks
settings = Settings
about = About

git-description = Git commit {$hash} on {$date}

# Menu items
new-disk-image = New Disk Image
attach-disk-image = Attach Disk Image
create-disk-from-drive = Create Disk From Drive
create-image = Create Image
restore-image-to-drive = Restore Image to Drive
restore-image = Restore Image
create-disk-from-partition = Create Disk Image From Partition
restore-image-to-partition = Restore Disk Image To Partition
image-file-path = Image file path
image-destination-path = Destination file path
image-source-path = Source image path
image-size = Image size
choose-path = Choose...
no-file-selected = No file selected
attach = Attach
restore-warning = This will overwrite the selected target device. This cannot be undone.
eject = Eject
eject-failed = Eject failed
power-off = Power Off
power-off-failed = Power off failed
format-disk = Format Disk
format-disk-failed = Format disk failed
smart-data-self-tests = SMART Data & Self-Tests
standby-now = Standby Now
standby-failed = Standby failed
wake-up-from-standby = Wake-up From Standby
wake-up-failed = Wake-up failed
unmount-failed = Unmount failed

# Unmount busy dialog
unmount-busy-title-template = {$device} is Busy
unmount-busy-message-template = The following processes are accessing {$mount}
unmount-busy-header-pid = PID
unmount-busy-header-command = Command
unmount-busy-header-user = User
unmount-busy-no-processes = Device is busy but no processes found. Try again or manually close any files.
unmount-busy-kill-warning = Killing processes may cause data loss or corruption.
unmount-busy-kill-and-retry = Kill Processes & Retry
retry = Retry

# Dialog buttons
ok = Ok
cancel = Cancel
continue = Continue
working = Working…

# Common
close = Close
refresh = Refresh
details = Details

# Format disk dialog
erase-dont-overwrite-quick = Don't Overwrite (Quick)
erase-overwrite-slow = Overwrite (Slow)
partitioning-dos-mbr = Legacy Compatible (DOS/MBR)
partitioning-gpt = Modern (GPT)
partitioning-none = None

# Create partition dialog
create-partition = Create Partition
create-partition-failed = Create partition failed
format-partition = Format Partition
format = Format
format-partition-description = This will format the selected volume. Size: { $size }
volume-name = Volume Name
partition-name = Partition Name
partition-size = Partition Size
free-space = Free Space
erase = Erase
password-protected = Password Protected
password = Password
confirm = Confirm
password-required = Password is required.
password-mismatch = Passwords do not match.
apply = Apply
untitled = Untitled

# Main view
no-disk-selected = No disk selected
no-volumes = No volumes available
partition-number = Partition { $number }
partition-number-with-name = Partition { $number }: { $name }
volumes = Volumes
unknown = Unknown
unresolved = Unresolved

# Info labels
size = Size
usage = Usage
mounted-at = Mounted at
contents = Contents
device = Device
partition = Partition
path = Path
uuid = UUID
model = Model
serial = Serial
partitioning = Partitioning
backing-file = Backing File

# Confirmation dialog
delete = Delete { $name }
delete-partition = Delete
delete-confirmation = Are you sure you wish to delete { $name }?
delete-failed = Delete failed

# Volume segments
free-space-segment = Free Space
reserved-space-segment = Reserved
filesystem = Filesystem
free-space-caption = Free space
reserved-space-caption = Reserved space

# Encrypted / LUKS
unlock-button = Unlock
lock = Lock
unlock = Unlock { $name }
passphrase = Passphrase
current-passphrase = Current passphrase
new-passphrase = New passphrase
change-passphrase = Change Passphrase
passphrase-mismatch = Passphrases do not match.
locked = Locked
unlocked = Unlocked
unlock-failed = Unlock failed
lock-failed = Lock failed
unlock-missing-partition = Could not find { $name } in the current device list.

# Volume commands
mount = Mount
unmount = Unmount
edit-mount-options = Edit Mount Options…
edit-encryption-options = Edit Encryption Options…
edit-partition = Edit Partition
edit = Edit
edit-partition-no-types = No partition types available for this partition table.
flag-legacy-bios-bootable = Legacy BIOS Bootable
flag-system-partition = System Partition
flag-hide-from-firmware = Hide from firmware
resize-partition = Resize Partition
resize = Resize
resize-partition-range = Allowed range: { $min } to { $max }
new-size = New Size
edit-filesystem = Edit Filesystem
label = Label
filesystem-label = Filesystem Label
check-filesystem = Check Filesystem
check-filesystem-warning = Checking a filesystem can take a long time. Continue?
repair-filesystem = Repair Filesystem
repair = Repair
repair-filesystem-warning = Repairing a filesystem can take a long time and may risk data loss. Continue?
take-ownership = Take Ownership
take-ownership-warning = This will change ownership of files to your user. This can take a long time and cannot be easily undone.
take-ownership-recursive = Apply recursively

# Mount/encryption options
user-session-defaults = User Session Defaults
mount-at-startup = Mount at system startup
unlock-at-startup = Unlock at system startup
require-auth-to-mount = Require authorization to mount or unmount
require-auth-to-unlock = Require authorization to unlock
show-in-ui = Show in user interface
identify-as = Identify As
other-options = Other options
mount-point = Mount point
filesystem-type = Filesystem type
display-name = Display name
icon-name = Icon name
symbolic-icon-name = Symbolic icon name
show-passphrase = Show passphrase
name = Name

# SMART
smart-no-data = No SMART data available.
smart-type = Type
smart-updated = Updated
smart-temperature = Temperature
smart-power-on-hours = Power-on hours
smart-selftest = Self-test
smart-selftest-short = Short self-test
smart-selftest-extended = Extended self-test
smart-selftest-abort = Abort self-test

# Volume types
lvm-logical-volume = LVM LV
lvm-physical-volume = LVM PV
luks-container = LUKS
partition-type = Partition
block-device = Device

# Status
not-mounted = Not mounted
can-create-partition = Can create partition

# Filesystem tools detection
fs-tools-missing-title = Missing Filesystem Tools
fs-tools-missing-desc = The following tools are not installed. Install them to enable full filesystem support:
fs-tools-all-installed-title = Filesystem Tools
fs-tools-all-installed = All filesystem tools are installed.
fs-tools-required-for = required for {$fs_name} support
offset = Offset

# Partition dialog labels
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

# Filesystem tools warning
fs-tools-warning = Some filesystem types are missing due to missing tools. See Settings for more info.
