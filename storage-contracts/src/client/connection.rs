// SPDX-License-Identifier: GPL-3.0-only

//! Shared D-Bus connection management
//!
//! This module provides a cached D-Bus system bus connection that is reused
//! across all client instances, improving performance by avoiding repeated
//! connection establishment.

use std::sync::OnceLock;

use zbus::Connection;

use super::error::ClientError;

/// Cached D-Bus system bus connection
static SYSTEM_CONNECTION: OnceLock<Connection> = OnceLock::new();

/// Get or create the shared system bus connection
///
/// The connection is established lazily on first use and cached for
/// subsequent calls. This avoids the overhead of creating multiple
/// D-Bus connections across different client instances.
pub async fn shared_connection() -> Result<&'static Connection, ClientError> {
    if let Some(conn) = SYSTEM_CONNECTION.get() {
        return Ok(conn);
    }

    // Race condition is acceptable - multiple connections during startup is fine
    // The OnceLock ensures only one connection is retained
    let conn = Connection::system()
        .await
        .map_err(|e| ClientError::Connection(format!("Failed to connect to system bus: {}", e)))?;

    // Ignore error if already set (another task won the race)
    let _ = SYSTEM_CONNECTION.set(conn);

    SYSTEM_CONNECTION.get().ok_or_else(|| {
        ClientError::Connection("Failed to initialize shared system bus connection".to_string())
    })
}
