// SPDX-License-Identifier: GPL-3.0-only

//! Disk power management operations

use std::collections::HashMap;
use anyhow::Result;
use udisks2::drive::DriveProxy;
use zbus::{Connection, zvariant::{OwnedObjectPath, Value}};

/// Helper to check if error indicates "not supported"
fn is_anyhow_not_supported(e: &anyhow::Error) -> bool {
    let msg = e.to_string();
    msg.contains("NotSupported")
        || msg.contains("not supported")
        || msg.contains("No such interface")
}

/// Helper to check if error indicates device is busy
fn is_anyhow_device_busy(e: &anyhow::Error) -> bool {
    let msg = e.to_string();
    msg.contains("DeviceBusy") || msg.contains("Device or resource busy")
}

/// Eject a drive
pub async fn eject_drive(drive_path: OwnedObjectPath, ejectable: bool) -> Result<()> {
    if !ejectable {
        return Err(anyhow::anyhow!("Not supported by this drive"));
    }

    let connection = Connection::system().await?;
    let proxy = DriveProxy::builder(&connection)
        .path(drive_path)?
        .build()
        .await?;

    match proxy.eject(HashMap::new()).await.map_err(Into::into) {
        Ok(()) => Ok(()),
        Err(e) if is_anyhow_not_supported(&e) => {
            Err(anyhow::anyhow!("Not supported by this drive"))
        }
        Err(e) if is_anyhow_device_busy(&e) => Err(anyhow::anyhow!(
            "Device is busy. Unmount any volumes on it and try again."
        )),
        Err(e) => Err(e),
    }
}

/// Power off a drive
pub async fn power_off_drive(drive_path: OwnedObjectPath, can_power_off: bool) -> Result<()> {
    if !can_power_off {
        return Err(anyhow::anyhow!("Not supported by this drive"));
    }

    let connection = Connection::system().await?;
    let proxy = DriveProxy::builder(&connection)
        .path(drive_path)?
        .build()
        .await?;
    
    proxy.power_off(HashMap::new()).await?;
    Ok(())
}

/// Put a drive into standby mode (spin down)
pub async fn standby_drive(drive_path: OwnedObjectPath) -> Result<()> {
    let connection = Connection::system().await?;
    let proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.UDisks2",
        drive_path.as_str(),
        "org.freedesktop.UDisks2.Drive.Ata",
    )
    .await?;

    let options: HashMap<&str, Value<'_>> = HashMap::new();
    let res: Result<()> = proxy
        .call("StandbyNow", &(options))
        .await
        .map_err(Into::into);
    
    match res {
        Ok(()) => Ok(()),
        Err(e) if is_anyhow_not_supported(&e) => {
            Err(anyhow::anyhow!("Not supported by this drive"))
        }
        Err(e) => Err(e),
    }
}

/// Wake up a drive from standby
pub async fn wakeup_drive(drive_path: OwnedObjectPath) -> Result<()> {
    let connection = Connection::system().await?;
    let proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.UDisks2",
        drive_path.as_str(),
        "org.freedesktop.UDisks2.Drive.Ata",
    )
    .await?;

    let options: HashMap<&str, Value<'_>> = HashMap::new();
    let res: Result<()> = proxy.call("Wakeup", &(options)).await.map_err(Into::into);
    
    match res {
        Ok(()) => Ok(()),
        Err(e) if is_anyhow_not_supported(&e) => {
            Err(anyhow::anyhow!("Not supported by this drive"))
        }
        Err(e) => Err(e),
    }
}

/// Remove a drive (loop device delete or removable drive power off)
pub async fn remove_drive(
    drive_path: OwnedObjectPath,
    block_path: &str,
    is_loop: bool,
    removable: bool,
    can_power_off: bool,
) -> Result<()> {
    let connection = Connection::system().await?;

    if is_loop {
        let proxy = zbus::Proxy::new(
            &connection,
            "org.freedesktop.UDisks2",
            block_path,
            "org.freedesktop.UDisks2.Loop",
        )
        .await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let res: Result<()> = proxy.call("Delete", &(options)).await.map_err(Into::into);
        
        match res {
            Ok(()) => Ok(()),
            Err(e) if is_anyhow_not_supported(&e) => Err(anyhow::anyhow!(
                "Remove not supported: device does not implement org.freedesktop.UDisks2.Loop"
            )),
            Err(e) if is_anyhow_device_busy(&e) => Err(anyhow::anyhow!(
                "Device is busy. Unmount any volumes on it and try again."
            )),
            Err(e) => Err(e),
        }
    } else if removable {
        // For removable drives, the expected "safe remove" behavior is power off.
        if !can_power_off {
            return Err(anyhow::anyhow!(
                "Remove not supported: drive is removable but does not support power off"
            ));
        }
        power_off_drive(drive_path, can_power_off).await
    } else {
        Err(anyhow::anyhow!(
            "Remove not supported: device is neither a loop-backed image nor a removable drive"
        ))
    }
}
