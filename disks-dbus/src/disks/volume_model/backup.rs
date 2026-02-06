use super::VolumeModel;
use anyhow::Result;

impl VolumeModel {
    pub async fn open_for_backup(&self) -> Result<std::os::fd::OwnedFd> {
        crate::disks::image::open_for_backup(self.path.clone()).await
    }

    pub async fn open_for_restore(&self) -> Result<std::os::fd::OwnedFd> {
        crate::disks::image::open_for_restore(self.path.clone()).await
    }

    pub async fn create_image(&self, _output_path: String) -> Result<()> {
        if self.connection.is_none() {
            return Err(crate::disks::DiskError::NotConnected(self.name.clone()).into());
        }
        Ok(())
    }
}
