use anyhow::Result;
use storage_models::VolumeKind;
use udisks2::encrypted::EncryptedProxy;
use zbus::Connection;

use super::model::DriveModel;
use crate::volume::node::{BlockIndex, VolumeNode};

impl DriveModel {
    pub(super) async fn build_volume_nodes_for_drive(
        &mut self,
        _connection: &Connection,
        _block_index: &BlockIndex,
    ) -> Result<()> {
        // TODO: Rewrite to build VolumeNode tree directly from UDisks2
        // without using intermediate VolumeModel flat list
        // For now, volumes tree will be empty
        self.volumes = vec![];
        Ok(())
    }
}
