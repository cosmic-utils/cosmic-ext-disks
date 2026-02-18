// SPDX-License-Identifier: GPL-3.0-only

//! COSMIC Ext Storage Service - D-Bus service for privileged disk operations
//!
//! This service provides a D-Bus interface for managing storage devices,
//! with Polkit-based authorization and socket activation support.

use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt};
use zbus::connection::Builder as ConnectionBuilder;

mod auth;
mod btrfs;
mod disks;
mod error;
mod filesystems;
mod image;
mod luks;
mod lvm;
mod partitions;
mod protected_paths;
mod rclone;
mod service;

use btrfs::BtrfsHandler;
use disks::DisksHandler;
use filesystems::FilesystemsHandler;
use image::ImageHandler;
use luks::LuksHandler;
use lvm::LVMHandler;
use partitions::PartitionsHandler;
use rclone::RcloneHandler;
use service::StorageService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to journald/stderr
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("storage_service=info,warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!(
        "Starting COSMIC Ext Storage Service v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Check if running as root
    if unsafe { libc::geteuid() } != 0 {
        tracing::error!("Storage service must run as root");
        anyhow::bail!("Service must run with root privileges");
    }

    // Build D-Bus connection with socket activation support
    // Create DisksHandler first so we can extract its manager for hotplug monitoring
    let disks_handler = DisksHandler::new().await?;
    let hotplug_manager = disks_handler.manager().clone();

    // Create RcloneHandler (rclone binary must be installed)
    let rclone_handler = match RcloneHandler::new() {
        Ok(handler) => Some(handler),
        Err(e) => {
            tracing::warn!("RClone not available: {}. RClone features disabled.", e);
            None
        }
    };

    let mut connection_builder = ConnectionBuilder::system()?
        .name("org.cosmic.ext.StorageService")?
        .serve_at("/org/cosmic/ext/StorageService", StorageService::new())?
        .serve_at("/org/cosmic/ext/StorageService/btrfs", BtrfsHandler::new())?
        .serve_at("/org/cosmic/ext/StorageService/disks", disks_handler)?
        .serve_at(
            "/org/cosmic/ext/StorageService/partitions",
            PartitionsHandler::new().await,
        )?
        .serve_at(
            "/org/cosmic/ext/StorageService/filesystems",
            FilesystemsHandler::new().await,
        )?
        .serve_at("/org/cosmic/ext/StorageService/lvm", LVMHandler::new())?
        .serve_at("/org/cosmic/ext/StorageService/luks", LuksHandler::new())?
        .serve_at("/org/cosmic/ext/StorageService/image", ImageHandler::new())?;

    // Conditionally serve RClone interface if available
    if let Some(handler) = rclone_handler {
        connection_builder =
            connection_builder.serve_at("/org/cosmic/ext/StorageService/rclone", handler)?;
    }

    let connection = connection_builder.build().await?;

    tracing::info!("Service registered on D-Bus system bus");
    tracing::info!("  - org.cosmic.ext.StorageService at /org/cosmic/ext/StorageService");
    tracing::info!("  - BTRFS interface at /org/cosmic/ext/StorageService/btrfs");
    tracing::info!("  - Disks interface at /org/cosmic/ext/StorageService/disks");
    tracing::info!("  - Partitions interface at /org/cosmic/ext/StorageService/partitions");
    tracing::info!("  - Filesystems interface at /org/cosmic/ext/StorageService/filesystems");
    tracing::info!("  - LVM interface at /org/cosmic/ext/StorageService/lvm");
    tracing::info!("  - LUKS interface at /org/cosmic/ext/StorageService/luks");
    tracing::info!("  - Image interface at /org/cosmic/ext/StorageService/image");
    tracing::info!("  - RClone interface at /org/cosmic/ext/StorageService/rclone");

    // Start disk hotplug monitoring
    disks::monitor_hotplug_events(
        connection.clone(),
        "/org/cosmic/ext/StorageService/disks",
        hotplug_manager,
    )
    .await?;
    tracing::info!("Disk hotplug monitoring enabled");

    // Keep service running until shutdown signal
    tracing::info!("Service ready, waiting for requests...");
    tokio::signal::ctrl_c().await?;
    tracing::info!("Received shutdown signal");

    tracing::info!("COSMIC Ext Storage Service shutting down");
    Ok(())
}
