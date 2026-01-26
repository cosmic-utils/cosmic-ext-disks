use anyhow::Result;
use udisks2::encrypted::EncryptedProxy;
use zbus::Connection;

use super::super::{BlockIndex, VolumeKind, VolumeNode};
use super::model::DriveModel;

impl DriveModel {
    pub(super) async fn build_volume_nodes_for_drive(
        &mut self,
        connection: &Connection,
        block_index: &BlockIndex,
    ) -> Result<()> {
        self.volumes = Vec::with_capacity(self.volumes_flat.len());
        for p in &self.volumes_flat {
            let label = if p.name.is_empty() {
                p.name()
            } else {
                p.name.clone()
            };

            // LUKS: treat as a container; children are cleartext filesystem or LVM PV.
            let encrypted_probe = EncryptedProxy::builder(connection).path(&p.path);

            let volume = if let Ok(builder) = encrypted_probe {
                match builder.build().await {
                    Ok(_) => match VolumeNode::crypto_container_for_partition(
                        connection,
                        p.path.clone(),
                        label.clone(),
                        block_index,
                    )
                    .await
                    {
                        Ok(v) => v,
                        Err(e) if DriveModel::is_missing_encrypted_interface(&e) => {
                            if p.id_type == "LVM2_member" {
                                VolumeNode::from_block_object(
                                    connection,
                                    p.path.clone(),
                                    label,
                                    VolumeKind::LvmPhysicalVolume,
                                    Some(block_index),
                                )
                                .await?
                            } else if p.has_filesystem {
                                VolumeNode::from_block_object(
                                    connection,
                                    p.path.clone(),
                                    label,
                                    VolumeKind::Filesystem,
                                    Some(block_index),
                                )
                                .await?
                            } else {
                                VolumeNode::from_block_object(
                                    connection,
                                    p.path.clone(),
                                    label,
                                    VolumeKind::Partition,
                                    Some(block_index),
                                )
                                .await?
                            }
                        }
                        Err(e) => return Err(e),
                    },
                    Err(_) => {
                        // Not actually encrypted; fall back below.
                        if p.id_type == "LVM2_member" {
                            VolumeNode::from_block_object(
                                connection,
                                p.path.clone(),
                                label,
                                VolumeKind::LvmPhysicalVolume,
                                Some(block_index),
                            )
                            .await?
                        } else if p.has_filesystem {
                            VolumeNode::from_block_object(
                                connection,
                                p.path.clone(),
                                label,
                                VolumeKind::Filesystem,
                                Some(block_index),
                            )
                            .await?
                        } else {
                            VolumeNode::from_block_object(
                                connection,
                                p.path.clone(),
                                label,
                                VolumeKind::Partition,
                                Some(block_index),
                            )
                            .await?
                        }
                    }
                }
            } else if p.id_type == "LVM2_member" {
                VolumeNode::from_block_object(
                    connection,
                    p.path.clone(),
                    label,
                    VolumeKind::LvmPhysicalVolume,
                    Some(block_index),
                )
                .await?
            } else if p.has_filesystem {
                VolumeNode::from_block_object(
                    connection,
                    p.path.clone(),
                    label,
                    VolumeKind::Filesystem,
                    Some(block_index),
                )
                .await?
            } else {
                VolumeNode::from_block_object(
                    connection,
                    p.path.clone(),
                    label,
                    VolumeKind::Partition,
                    Some(block_index),
                )
                .await?
            };

            self.volumes.push(volume);
        }

        Ok(())
    }
}
