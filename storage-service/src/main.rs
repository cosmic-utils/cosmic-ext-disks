// SPDX-License-Identifier: GPL-3.0-only

//! COSMIC Ext Storage Service - D-Bus service for privileged disk operations
//!
//! This service provides a D-Bus interface for managing storage devices,
//! with Polkit-based authorization and socket activation support.

use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt};
use zbus::connection::Builder as ConnectionBuilder;

mod adapters;
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
mod routing;
mod service;

use btrfs::BtrfsHandler;
use disks::DisksHandler;
use filesystems::FilesystemsHandler;
use image::ImageHandler;
use luks::LuksHandler;
use lvm::LVMHandler;
use partitions::PartitionsHandler;
use rclone::RcloneHandler;
use routing::{AdapterRegistry, Concern};
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

    // Build fixed adapter routing at startup and fail fast if required concerns are missing.
    let adapters = AdapterRegistry::build_default().await?;
    tracing::info!(
        "Adapter routing: Disks -> {}, Partitions -> {}, Filesystems -> {}, Luks -> {}, Image -> {}",
        adapters.route_for(Concern::Disks).unwrap_or("<missing>"),
        adapters
            .route_for(Concern::Partitions)
            .unwrap_or("<missing>"),
        adapters
            .route_for(Concern::Filesystems)
            .unwrap_or("<missing>"),
        adapters.route_for(Concern::Luks).unwrap_or("<missing>"),
        adapters.route_for(Concern::Image).unwrap_or("<missing>")
    );

    // Build D-Bus connection with socket activation support
    let disks_handler = DisksHandler::new(adapters.disk_query(), adapters.disk_ops());

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
        .serve_at("/org/cosmic/ext/Storage/Service/disks", disks_handler)?
        .serve_at(
            "/org/cosmic/ext/Storage/Service/partitions",
            PartitionsHandler::new(adapters.partition_ops()),
        )?
        .serve_at(
            "/org/cosmic/ext/Storage/Service/filesystems",
            FilesystemsHandler::new(adapters.filesystem_ops())?,
        )?
        .serve_at("/org/cosmic/ext/Storage/Service/lvm", LVMHandler::new())?
        .serve_at(
            "/org/cosmic/ext/Storage/Service/luks",
            LuksHandler::new(adapters.luks_ops()),
        )?
        .serve_at(
            "/org/cosmic/ext/Storage/Service/image",
            ImageHandler::new(adapters.image_ops()),
        )?;

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
    tracing::info!("  - LVM interface at /org/cosmic/ext/Storage/Service/lvm");
    tracing::info!("  - LUKS interface at /org/cosmic/ext/Storage/Service/luks");
    tracing::info!("  - Image interface at /org/cosmic/ext/Storage/Service/image");
    tracing::info!("  - RClone interface at /org/cosmic/ext/Storage/Service/rclone");

    // Start disk hotplug monitoring
    disks::monitor_hotplug_events(
        connection.clone(),
        "/org/cosmic/ext/Storage/Service/disks",
        adapters.disk_query(),
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
