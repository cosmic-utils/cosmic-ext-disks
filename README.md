# 🚧 Disks 🚧

> [!WARNING]
> This is now stable enough to use for disk operations, however it is still in early development so use with caution!
---
A Disk utility application for the Cosmic Desktop.


### Prerequisites
You will need the following packages/services:
 - `udisks2` (system service; required for device enumeration and events)

For partition type support:
 - `ntfs-3g`
 - `exfatprogs`
 - `dosfstools`


### What Works
 * Read disk info (Slight issues with offset)
 * Delete partition 
 * Create Partition 
    * Both GPT/MBR supported
    * EXT4, vFAT, extFAT & NTFS tested so far

I am currently actively developing this again after a 5 month hiatus, so this list should be getting longer quite regularly from now on.

### What doesn't work
Everything else!



### Future Plans

#### Better UI/UX
The UI of disks is essentially a clone of Gnome Disks at the moment. There are plans to focus on this and improve it once the lower-level functionality is somewhat complete.

![Screenshot of cosmos-disks](https://github.com/stoorps/cosmos-apps/blob/main/screenshots/cosmos-disks.png)


### Project structure

#### disks-ui
The application.

#### disks-dbus
This project is an abstraction layer for dbus interfaces. The idea here is to provide models that can easily be swapped out at a later date, as better suited rust crates become available for achieving the same functionality.
