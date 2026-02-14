// SPDX-License-Identifier: GPL-3.0-only

//! COSMIC Storage Service - D-Bus service for privileged disk operations
//! 
//! This service provides a D-Bus interface for managing storage devices,
//! with Polkit-based authorization and socket activation support.

use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};
use zbus::connection::Builder as ConnectionBuilder;

mod auth;
mod btrfs;
mod conversions;
mod disks;
mod error;
mod partitions;
mod service;

use btrfs::BtrfsHandler;
use disks::DisksHandler;
use partitions::PartitionsHandler;
use service::StorageService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to journald/stderr
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("storage_service=info,warn"))
        )
        .with_writer(std::io::stderr)
        .init();
    
    tracing::info!("Starting COSMIC Storage Service v{}", env!("CARGO_PKG_VERSION"));
    
    // Check if running as root
    if unsafe { libc::geteuid() } != 0 {
        tracing::error!("Storage service must run as root");
        anyhow::bail!("Service must run with root privileges");
    }
    
    // Build D-Bus connection with socket activation support
    let connection = ConnectionBuilder::system()?
        .name("org.cosmic.ext.StorageService")?
        .serve_at("/org/cosmic/ext/StorageService", StorageService::new())?
        .serve_at("/org/cosmic/ext/StorageService/btrfs", BtrfsHandler::new())?
        .serve_at(
            "/org/cosmic/ext/StorageService/disks",
            DisksHandler::new().await?,
        )?
        .serve_at(
            "/org/cosmic/ext/StorageService/partitions",
            PartitionsHandler::new(),
        )?
        .build()
        .await?;
    
    tracing::info!("Service registered on D-Bus system bus");
    tracing::info!("  - org.cosmic.ext.StorageService at /org/cosmic/ext/StorageService");
    tracing::info!("  - BTRFS interface at /org/cosmic/ext/StorageService/btrfs");
    tracing::info!("  - Disks interface at /org/cosmic/ext/StorageService/disks");
    tracing::info!("  - Partitions interface at /org/cosmic/ext/StorageService/partitions");
    
    // Start disk hotplug monitoring
    disks::monitor_hotplug_events(
        connection.clone(),
        "/org/cosmic/ext/StorageService/disks",
    )
    .await?;
    tracing::info!("Disk hotplug monitoring enabled");
    
    // Keep service running until shutdown signal
    tracing::info!("Service ready, waiting for requests...");
    tokio::signal::ctrl_c().await?;
    tracing::info!("Received shutdown signal");
    
    tracing::info!("COSMIC Storage Service shutting down");
    Ok(())
}
