// SPDX-License-Identifier: GPL-3.0-only

//! COSMIC Ext Storage Service - D-Bus service for privileged disk operations
//!
//! This service provides a D-Bus interface for managing storage devices,
//! with Polkit-based authorization and socket activation support.

use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt};
use zbus::connection::Builder as ConnectionBuilder;

mod auth;
mod error;
mod handlers;
mod policies;
mod protected_paths;

use handlers::btrfs::BtrfsHandler;
use handlers::disk::DiskHandler;
use handlers::filesystem::FilesystemHandler;
use handlers::image::ImageHandler;
use handlers::logical::LogicalHandler;
use handlers::luks::LuksHandler;
use handlers::lvm::LvmHandler;
use handlers::partition::PartitionHandler;
use handlers::rclone::RcloneHandler;
use handlers::service::StorageService;

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
    let disk_handler = DiskHandler::new();

    // Create RcloneHandler (rclone binary must be installed)
    let rclone_handler = match RcloneHandler::new() {
        Ok(handler) => Some(handler),
        Err(e) => {
            tracing::warn!("RClone not available: {}. RClone features disabled.", e);
            None
        }
    };

    let mut connection_builder = ConnectionBuilder::system()?
        .name("org.cosmic.ext.Storage.Service")?
        .serve_at("/org/cosmic/ext/Storage/Service", StorageService::new())?
        .serve_at("/org/cosmic/ext/Storage/Service/btrfs", BtrfsHandler::new())?
        .serve_at("/org/cosmic/ext/Storage/Service/disks", disk_handler)?
        .serve_at(
            "/org/cosmic/ext/Storage/Service/partitions",
            PartitionHandler::new(),
        )?
        .serve_at(
            "/org/cosmic/ext/Storage/Service/filesystems",
            FilesystemHandler::new()?,
        )?
        .serve_at(
            "/org/cosmic/ext/Storage/Service/logical",
            LogicalHandler::new(),
        )?
        .serve_at("/org/cosmic/ext/Storage/Service/lvm", LvmHandler::new())?
        .serve_at("/org/cosmic/ext/Storage/Service/luks", LuksHandler::new())?
        .serve_at("/org/cosmic/ext/Storage/Service/image", ImageHandler::new())?;

    // Conditionally serve RClone interface if available
    if let Some(handler) = rclone_handler {
        connection_builder =
            connection_builder.serve_at("/org/cosmic/ext/Storage/Service/rclone", handler)?;
    }

    let connection = connection_builder.build().await?;

    tracing::info!("Service registered on D-Bus system bus");
    tracing::info!("  - org.cosmic.ext.Storage.Service at /org/cosmic/ext/Storage/Service");
    tracing::info!("  - BTRFS interface at /org/cosmic/ext/Storage/Service/btrfs");
    tracing::info!("  - Disks interface at /org/cosmic/ext/Storage/Service/disks");
    tracing::info!("  - Partitions interface at /org/cosmic/ext/Storage/Service/partitions");
    tracing::info!("  - Filesystems interface at /org/cosmic/ext/Storage/Service/filesystems");
    tracing::info!("  - Logical interface at /org/cosmic/ext/Storage/Service/logical");
    tracing::info!("  - LVM interface at /org/cosmic/ext/Storage/Service/lvm");
    tracing::info!("  - LUKS interface at /org/cosmic/ext/Storage/Service/luks");
    tracing::info!("  - Image interface at /org/cosmic/ext/Storage/Service/image");
    tracing::info!("  - RClone interface at /org/cosmic/ext/Storage/Service/rclone");

    // Start disk hotplug monitoring
    handlers::disk::hotplug::monitor_hotplug_events(
        connection.clone(),
        "/org/cosmic/ext/Storage/Service/disks",
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
