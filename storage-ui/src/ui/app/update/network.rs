// SPDX-License-Identifier: GPL-3.0-only

//! Network mount message handling

use super::super::message::Message;
use super::super::state::AppModel;
use crate::client::RcloneClient;
use crate::ui::network::NetworkMessage;
use cosmic::app::Task;
use storage_common::rclone::{ConfigScope, MountStatus};

/// Handle network-related messages
pub(crate) fn handle_network_message(app: &mut AppModel, message: NetworkMessage) -> Task<Message> {
    match message {
        NetworkMessage::LoadRemotes => {
            app.network.loading = true;
            return Task::perform(
                async {
                    match RcloneClient::new().await {
                        Ok(client) => match client.list_remotes().await {
                            Ok(list) => Ok(list.remotes),
                            Err(e) => Err(format!("Failed to list remotes: {}", e)),
                        },
                        Err(e) => Err(format!("RClone not available: {}", e)),
                    }
                },
                |result| Message::NetworkRemotesLoaded(result).into(),
            );
        }

        NetworkMessage::RemotesLoaded(result) => {
            app.network.loading = false;
            match result {
                Ok(remotes) => {
                    app.network.rclone_available = true;
                    app.network.set_remotes(remotes);
                }
                Err(e) => {
                    app.network.rclone_available = false;
                    app.network.error = Some(e);
                }
            }
        }

        NetworkMessage::SelectRemote { name, scope } => {
            app.network.select(Some(name), Some(scope));
        }

        NetworkMessage::MountRemote { name, scope } => {
            app.network.set_loading(&name, scope, true);
            let name_for_task = name.clone();
            return Task::perform(
                async move {
                    match RcloneClient::new().await {
                        Ok(client) => {
                            match client.mount(&name_for_task, &scope.to_string()).await {
                                Ok(()) => Ok(()),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        Err(e) => Err(e.to_string()),
                    }
                },
                move |result| {
                    Message::Network(NetworkMessage::MountCompleted {
                        name: name.clone(),
                        scope,
                        result,
                    })
                    .into()
                },
            );
        }

        NetworkMessage::UnmountRemote { name, scope } => {
            app.network.set_loading(&name, scope, true);
            let name_for_task = name.clone();
            return Task::perform(
                async move {
                    match RcloneClient::new().await {
                        Ok(client) => {
                            match client.unmount(&name_for_task, &scope.to_string()).await {
                                Ok(()) => Ok(()),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        Err(e) => Err(e.to_string()),
                    }
                },
                move |result| {
                    Message::Network(NetworkMessage::UnmountCompleted {
                        name: name.clone(),
                        scope,
                        result,
                    })
                    .into()
                },
            );
        }

        NetworkMessage::RestartRemote { name, scope } => {
            // Restart is implemented as unmount followed by mount
            // Set loading state and start with unmount
            app.network.set_loading(&name, scope, true);
            let name_for_task = name.clone();
            return Task::perform(
                async move {
                    match RcloneClient::new().await {
                        Ok(client) => {
                            // First unmount
                            if let Err(e) = client.unmount(&name_for_task, &scope.to_string()).await
                            {
                                // If unmount fails, try to mount anyway (might not have been mounted)
                                tracing::warn!(
                                    "Unmount during restart failed: {}, attempting mount anyway",
                                    e
                                );
                            }
                            // Then mount
                            match client.mount(&name_for_task, &scope.to_string()).await {
                                Ok(()) => Ok(()),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        Err(e) => Err(e.to_string()),
                    }
                },
                move |result| {
                    Message::Network(NetworkMessage::MountCompleted {
                        name: name.clone(),
                        scope,
                        result,
                    })
                    .into()
                },
            );
        }

        NetworkMessage::MountCompleted {
            name,
            scope,
            result,
        } => match result {
            Ok(()) => {
                app.network
                    .set_mount_status(&name, scope, MountStatus::Mounted);
            }
            Err(e) => {
                app.network
                    .set_mount_status(&name, scope, MountStatus::Error(e.clone()));
                app.network.set_error(&name, scope, Some(e));
            }
        },

        NetworkMessage::UnmountCompleted {
            name,
            scope,
            result,
        } => match result {
            Ok(()) => {
                app.network
                    .set_mount_status(&name, scope, MountStatus::Unmounted);
            }
            Err(e) => {
                app.network
                    .set_mount_status(&name, scope, MountStatus::Error(e.clone()));
                app.network.set_error(&name, scope, Some(e));
            }
        },

        NetworkMessage::TestRemote { name, scope } => {
            let name_for_task = name.clone();
            return Task::perform(
                async move {
                    match RcloneClient::new().await {
                        Ok(client) => {
                            match client.test_remote(&name_for_task, &scope.to_string()).await {
                                Ok(result) => {
                                    if result.success {
                                        Ok(format!(
                                            "Connection successful{}",
                                            result
                                                .latency_ms
                                                .map(|l| format!(" ({}ms)", l))
                                                .unwrap_or_default()
                                        ))
                                    } else {
                                        Err(result.message)
                                    }
                                }
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        Err(e) => Err(e.to_string()),
                    }
                },
                move |result| {
                    Message::Network(NetworkMessage::TestCompleted {
                        name: name.clone(),
                        result,
                    })
                    .into()
                },
            );
        }

        NetworkMessage::TestCompleted { name: _, result } => {
            // TODO: Show result in a dialog
            match result {
                Ok(msg) => tracing::info!("Test result: {}", msg),
                Err(e) => tracing::error!("Test failed: {}", e),
            }
        }

        NetworkMessage::RefreshStatus { name, scope } => {
            let name_for_task = name.clone();
            return Task::perform(
                async move {
                    match RcloneClient::new().await {
                        Ok(client) => {
                            match client
                                .get_mount_status(&name_for_task, &scope.to_string())
                                .await
                            {
                                Ok(status) => Ok(status.status.is_mounted()),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        Err(e) => Err(e.to_string()),
                    }
                },
                move |result| {
                    // Default to unmounted on error
                    let mounted = result.unwrap_or(false);
                    Message::Network(NetworkMessage::StatusRefreshed {
                        name: name.clone(),
                        scope,
                        mounted,
                    })
                    .into()
                },
            );
        }

        NetworkMessage::StatusRefreshed {
            name,
            scope,
            mounted,
        } => {
            let status = if mounted {
                MountStatus::Mounted
            } else {
                MountStatus::Unmounted
            };
            app.network.set_mount_status(&name, scope, status);
        }

        NetworkMessage::OpenAddRemote => {
            // TODO: Open add remote dialog
            tracing::info!("Open add remote dialog");
        }

        NetworkMessage::OpenEditRemote { name: _, scope: _ } => {
            // TODO: Open edit remote dialog
            tracing::info!("Open edit remote dialog");
        }

        NetworkMessage::DeleteRemote { name, scope } => {
            // TODO: Show confirmation dialog first
            // For now, just delete directly
            let name_for_task = name.clone();
            return Task::perform(
                async move {
                    match RcloneClient::new().await {
                        Ok(client) => {
                            match client
                                .delete_remote(&name_for_task, &scope.to_string())
                                .await
                            {
                                Ok(()) => Ok(()),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        Err(e) => Err(e.to_string()),
                    }
                },
                move |result| {
                    Message::Network(NetworkMessage::DeleteCompleted {
                        name: name.clone(),
                        result,
                    })
                    .into()
                },
            );
        }

        NetworkMessage::DeleteRemoteConfirmed { name, scope } => {
            let name_for_task = name.clone();
            return Task::perform(
                async move {
                    match RcloneClient::new().await {
                        Ok(client) => {
                            match client
                                .delete_remote(&name_for_task, &scope.to_string())
                                .await
                            {
                                Ok(()) => Ok(()),
                                Err(e) => Err(e.to_string()),
                            }
                        }
                        Err(e) => Err(e.to_string()),
                    }
                },
                move |result| {
                    Message::Network(NetworkMessage::DeleteCompleted {
                        name: name.clone(),
                        result,
                    })
                    .into()
                },
            );
        }

        NetworkMessage::DeleteCompleted { name, result } => {
            match result {
                Ok(()) => {
                    // Remove from state
                    app.network
                        .mounts
                        .remove(&(name.clone(), ConfigScope::User));
                    app.network
                        .mounts
                        .remove(&(name.clone(), ConfigScope::System));
                    tracing::info!("Deleted remote: {}", name);
                }
                Err(e) => {
                    tracing::error!("Failed to delete remote {}: {}", name, e);
                }
            }
        }

        NetworkMessage::Cancel => {
            // Cancel any pending operation
        }
    }

    Task::none()
}
