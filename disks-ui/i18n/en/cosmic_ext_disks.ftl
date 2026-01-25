app-title = Disks
about = About
view = View
welcome = Welcome to COSMIC! ✨
page-id = Page { $num }
git-description = Git commit {$hash} on {$date}

# Menu items
menu-image = Image
menu-disk = Disk
menu-view = View
new-disk-image = New Disk Image
attach-disk-image = Attach Disk Image
create-disk-from-drive = Create Disk From Drive
restore-image-to-drive = Restore Image to Drive
create-disk-from-partition = Create Disk Image From Partition
restore-image-to-partition = Restore Disk Image To Partition
image-file-path = Image file path
image-destination-path = Destination file path
image-source-path = Source image path
image-size = Image size
create-image = Create image
restore-image = Restore image
attach = Attach
restore-warning = This will overwrite the selected target device. This cannot be undone.
eject = Eject
power-off = Power Off
format-disk = Format Disk
benchmark-disk = Benchmark Disk
smart-data-self-tests = SMART Data & Self-Tests
drive-settings = Drive Settings
standby-now = Standby Now
wake-up-from-standby = Wake-up From Standby

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
format-partition = Format Partition
format-partition-description = This will format the selected volume. Size: { $size }
volume-name = Volume Name
partition-name = Partition Name
partition-size = Partition Size
free-space = Free Space
erase = Erase
password-protected = Password Protected
password = Password
confirm = Confirm
apply = Apply
untitled = Untitled

# Main view
no-disk-selected = No disk selected
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

# Confirmation dialog
delete = Delete { $name }
delete-confirmation = Are you sure you wish to delete { $name }?
delete-failed = Delete failed

# Volume segments
free-space-segment = Free Space
reserved-space-segment = Reserved
filesystem = Filesystem
free-space-caption = Free space
reserved-space-caption = Reserved space

# Volumes view
show-reserved = Show reserved

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
mount-toggle = Mount / Unmount
edit-mount-options = Edit Mount Options…
edit-encryption-options = Edit Encryption Options…
edit-partition = Edit Partition
edit-partition-no-types = No partition types available for this partition table.
flag-legacy-bios-bootable = Legacy BIOS Bootable
flag-system-partition = System Partition
flag-hide-from-firmware = Hide from firmware
resize-partition = Resize Partition
resize-partition-range = Allowed range: { $min } to { $max }
new-size = New Size
edit-filesystem = Edit Filesystem
filesystem-label = Filesystem Label
check-filesystem = Check Filesystem
check-filesystem-warning = Checking a filesystem can take a long time. Continue?
repair-filesystem = Repair Filesystem
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
