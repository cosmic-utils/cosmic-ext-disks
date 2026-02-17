// SPDX-License-Identifier: GPL-3.0-only

//! Network mount message handling

use super::super::message::Message;
use super::super::state::AppModel;
use crate::client::RcloneClient;
use crate::ui::dialogs::message::RemoteConfigDialogMessage;
use crate::ui::dialogs::state::{RemoteConfigDialog, ShowDialog};
use crate::ui::network::NetworkMessage;
use cosmic::app::Task;
use storage_common::rclone::{ConfigScope, MountStatus, SUPPORTED_REMOTE_TYPES};

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
                app.network.set_error(&name, scope, Some(e.clone()));
                // Show error dialog
                app.dialog = Some(ShowDialog::Info {
                    title: "Mount Failed".to_string(),
                    body: format!("Failed to mount remote '{}': {}", name, e),
                });
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
                app.network.set_error(&name, scope, Some(e.clone()));
                // Show error dialog
                app.dialog = Some(ShowDialog::Info {
                    title: "Unmount Failed".to_string(),
                    body: format!("Failed to unmount remote '{}': {}", name, e),
                });
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
            // Show test result in a dialog
            let (title, body) = match result {
                Ok(msg) => ("Connection Test".to_string(), msg),
                Err(e) => ("Connection Test Failed".to_string(), e),
            };
            app.dialog = Some(ShowDialog::Info { title, body });
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
            // Open the add remote dialog
            app.dialog = Some(ShowDialog::RemoteConfig(RemoteConfigDialog {
                name: String::new(),
                remote_type: "drive".to_string(),
                remote_type_index: 0,
                scope: ConfigScope::User,
                is_edit: false,
                original_name: None,
                running: false,
                error: None,
            }));
        }

        NetworkMessage::OpenEditRemote { name, scope } => {
            // Find the remote in state to get its type
            let remote_type = app
                .network
                .get_mount(&name, scope)
                .map(|m| m.config.remote_type.clone())
                .unwrap_or_else(|| "drive".to_string());

            let remote_type_index = SUPPORTED_REMOTE_TYPES
                .iter()
                .position(|&t| t == remote_type)
                .unwrap_or(0);

            // Open the edit remote dialog
            app.dialog = Some(ShowDialog::RemoteConfig(RemoteConfigDialog {
                name: name.clone(),
                remote_type: remote_type.clone(),
                remote_type_index,
                scope,
                is_edit: true,
                original_name: Some(name.clone()),
                running: false,
                error: None,
            }));
        }

        NetworkMessage::DeleteRemote { name, scope } => {
            // Show confirmation dialog before deleting
            app.dialog = Some(ShowDialog::ConfirmDeleteRemote {
                name,
                scope,
            });
        }

        NetworkMessage::ConfirmDeleteRemote { name, scope } => {
            // Close the dialog first
            app.dialog = None;
            // Then proceed with the actual delete
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
                    // Show error dialog
                    app.dialog = Some(ShowDialog::Info {
                        title: "Delete Failed".to_string(),
                        body: format!("Failed to delete remote '{}': {}", name, e),
                    });
                }
            }
        }

        NetworkMessage::Cancel => {
            // Cancel any pending operation
        }
    }

    Task::none()
}

/// Handle remote config dialog messages
pub(crate) fn handle_remote_config_dialog(
    app: &mut AppModel,
    message: RemoteConfigDialogMessage,
) -> Task<Message> {
    let dialog = match &mut app.dialog {
        Some(ShowDialog::RemoteConfig(d)) => d,
        _ => return Task::none(),
    };

    match message {
        RemoteConfigDialogMessage::NameUpdate(name) => {
            dialog.name = name;
        }

        RemoteConfigDialogMessage::RemoteTypeIndexUpdate(index) => {
            dialog.remote_type_index = index;
            if let Some(remote_type) = SUPPORTED_REMOTE_TYPES.get(index) {
                dialog.remote_type = remote_type.to_string();
            }
        }

        RemoteConfigDialogMessage::ScopeUpdate(index) => {
            dialog.scope = match index {
                0 => ConfigScope::User,
                _ => ConfigScope::System,
            };
        }

        RemoteConfigDialogMessage::Save => {
            // Validate name
            if dialog.name.trim().is_empty() {
                dialog.error = Some("Remote name cannot be empty".to_string());
                return Task::none();
            }

            let name = dialog.name.clone();
            let remote_type = dialog.remote_type.clone();
            let scope = dialog.scope;
            let is_edit = dialog.is_edit;
            let original_name = dialog.original_name.clone();

            dialog.running = true;
            dialog.error = None;

            return Task::perform(
                async move {
                    let client = RcloneClient::new().await.map_err(|e| e.to_string())?;
                    let config = storage_common::rclone::RemoteConfig::new(
                        name.clone(),
                        remote_type,
                        scope,
                    );

                    if is_edit {
                        // For edit, we need to delete the old remote and create a new one
                        // if the name changed
                        if let Some(original) = &original_name
                            && original != &name
                        {
                            client.delete_remote(original, &scope.to_string()).await.map_err(|e| e.to_string())?;
                        }
                        client.update_remote(&name, &config).await.map_err(|e| e.to_string())?;
                    } else {
                        client.create_remote(&config).await.map_err(|e| e.to_string())?;
                    }
                    Ok(())
                },
                |result: Result<(), String>| {
                    Message::RemoteConfigDialog(RemoteConfigDialogMessage::Complete(result)).into()
                },
            );
        }

        RemoteConfigDialogMessage::Cancel => {
            app.dialog = None;
        }

        RemoteConfigDialogMessage::Complete(result) => {
            dialog.running = false;
            match result {
                Ok(()) => {
                    app.dialog = None;
                    // Refresh the remote list
                    return Task::done(Message::Network(NetworkMessage::LoadRemotes).into());
                }
                Err(e) => {
                    dialog.error = Some(e);
                }
            }
        }
    }

    Task::none()
}
