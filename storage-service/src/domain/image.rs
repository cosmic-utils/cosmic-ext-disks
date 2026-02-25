// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

pub trait ImageDomain: Send + Sync {
    fn validate_output_path_parent_exists(&self, output_path: &str) -> zbus::fdo::Result<()>;
}

pub struct DefaultImageDomain;

impl ImageDomain for DefaultImageDomain {
    fn validate_output_path_parent_exists(&self, output_path: &str) -> zbus::fdo::Result<()> {
        let output_path_obj = Path::new(output_path);
        if let Some(parent) = output_path_obj.parent()
            && !parent.exists()
        {
            return Err(zbus::fdo::Error::Failed(format!(
                "Output directory does not exist: {}",
                parent.display()
            )));
        }

        Ok(())
    }
}
