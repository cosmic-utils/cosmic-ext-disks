// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;
use std::sync::Arc;

use crate::adapters::udisks::build_default_adapters;
use anyhow::{Result, anyhow};
use storage_contracts::{
    DiskOpsAdapter, DiskQueryAdapter, FilesystemOpsAdapter, ImageOpsAdapter, LuksOpsAdapter,
    PartitionOpsAdapter,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Concern {
    Disks,
    Partitions,
    Filesystems,
    Luks,
    Image,
}

const REQUIRED_CONCERNS: [Concern; 5] = [
    Concern::Disks,
    Concern::Partitions,
    Concern::Filesystems,
    Concern::Luks,
    Concern::Image,
];

pub struct AdapterRegistry {
    routes: HashMap<Concern, &'static str>,
    disk_query: Arc<dyn DiskQueryAdapter>,
    disk_ops: Arc<dyn DiskOpsAdapter>,
    partition_ops: Arc<dyn PartitionOpsAdapter>,
    filesystem_ops: Arc<dyn FilesystemOpsAdapter>,
    luks_ops: Arc<dyn LuksOpsAdapter>,
    image_ops: Arc<dyn ImageOpsAdapter>,
}

impl AdapterRegistry {
    pub async fn build_default() -> Result<Self> {
        let adapters = build_default_adapters().await?;

        let mut routes = HashMap::new();
        routes.insert(Concern::Disks, "udisks");
        routes.insert(Concern::Partitions, "udisks");
        routes.insert(Concern::Filesystems, "udisks");
        routes.insert(Concern::Luks, "udisks");
        routes.insert(Concern::Image, "udisks");

        for concern in REQUIRED_CONCERNS {
            if !routes.contains_key(&concern) {
                return Err(anyhow!(
                    "Missing required adapter routing for concern: {:?}",
                    concern
                ));
            }
        }

        Ok(Self {
            routes,
            disk_query: adapters.disk_query,
            disk_ops: adapters.disk_ops,
            partition_ops: adapters.partition_ops,
            filesystem_ops: adapters.filesystem_ops,
            luks_ops: adapters.luks_ops,
            image_ops: adapters.image_ops,
        })
    }

    pub fn disk_query(&self) -> Arc<dyn DiskQueryAdapter> {
        self.disk_query.clone()
    }

    pub fn disk_ops(&self) -> Arc<dyn DiskOpsAdapter> {
        self.disk_ops.clone()
    }

    pub fn partition_ops(&self) -> Arc<dyn PartitionOpsAdapter> {
        self.partition_ops.clone()
    }

    pub fn filesystem_ops(&self) -> Arc<dyn FilesystemOpsAdapter> {
        self.filesystem_ops.clone()
    }

    pub fn luks_ops(&self) -> Arc<dyn LuksOpsAdapter> {
        self.luks_ops.clone()
    }

    pub fn image_ops(&self) -> Arc<dyn ImageOpsAdapter> {
        self.image_ops.clone()
    }

    pub fn route_for(&self, concern: Concern) -> Option<&'static str> {
        self.routes.get(&concern).copied()
    }
}
