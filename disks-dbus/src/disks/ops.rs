use std::collections::HashMap;

use anyhow::Result;
use futures::future::BoxFuture;
use udisks2::{
    block::BlockProxy, filesystem::FilesystemProxy, partition::PartitionProxy,
    partitiontable::PartitionTableProxy,
};
use zbus::{Connection, zvariant::OwnedObjectPath};

use super::ByteRange;
use crate::{COMMON_DOS_TYPES, COMMON_GPT_TYPES, CreatePartitionInfo, PartitionTypeInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CreatePartitionAndFormatArgs {
    pub(crate) table_block_path: String,
    pub(crate) table_type: String,
    pub(crate) offset: u64,
    pub(crate) size: u64,
    pub(crate) partition_type: String,
    pub(crate) create_name: String,
    pub(crate) create_partition_kind: Option<String>,
    pub(crate) filesystem_type: String,
    pub(crate) erase: bool,
    pub(crate) label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PartitionFormatArgs {
    pub(crate) block_path: OwnedObjectPath,
    pub(crate) filesystem_type: String,
    pub(crate) erase: bool,
    pub(crate) label: Option<String>,
}

pub(crate) trait DiskBackend: Send + Sync {
    fn create_partition_and_format(
        &self,
        args: CreatePartitionAndFormatArgs,
    ) -> BoxFuture<'_, Result<()>>;

    fn fs_mount(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>>;
    fn fs_unmount(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>>;
    fn partition_delete(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>>;
    fn block_format(&self, args: PartitionFormatArgs) -> BoxFuture<'_, Result<()>>;
}

#[derive(Clone)]
pub(crate) struct RealDiskBackend {
    connection: Connection,
}

impl RealDiskBackend {
    pub(crate) fn new(connection: Connection) -> Self {
        Self { connection }
    }
}

impl DiskBackend for RealDiskBackend {
    fn create_partition_and_format(
        &self,
        args: CreatePartitionAndFormatArgs,
    ) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let proxy = PartitionTableProxy::builder(&self.connection)
                .path(args.table_block_path.clone())?
                .build()
                .await?;

            let mut create_options = HashMap::new();
            if let Some(kind) = args.create_partition_kind.as_deref() {
                create_options.insert("partition-type", zbus::zvariant::Value::from(kind));
            }

            let mut format_options = HashMap::new();
            if args.erase {
                format_options.insert("erase", zbus::zvariant::Value::from("zero"));
            }
            if let Some(label) = args.label.as_deref()
                && !label.is_empty()
            {
                format_options.insert("label", zbus::zvariant::Value::from(label));
            }

            proxy
                .create_partition_and_format(
                    args.offset,
                    args.size,
                    args.partition_type.as_str(),
                    args.create_name.as_str(),
                    create_options,
                    args.filesystem_type.as_str(),
                    format_options,
                )
                .await?;

            Ok(())
        })
    }

    fn fs_mount(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let proxy = FilesystemProxy::builder(&self.connection)
                .path(&path)?
                .build()
                .await?;
            proxy.mount(HashMap::new()).await?;
            Ok(())
        })
    }

    fn fs_unmount(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let proxy = FilesystemProxy::builder(&self.connection)
                .path(&path)?
                .build()
                .await?;
            proxy.unmount(HashMap::new()).await?;
            Ok(())
        })
    }

    fn partition_delete(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let proxy = PartitionProxy::builder(&self.connection)
                .path(&path)?
                .build()
                .await?;
            proxy.delete(HashMap::new()).await?;
            Ok(())
        })
    }

    fn block_format(&self, args: PartitionFormatArgs) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let proxy = BlockProxy::builder(&self.connection)
                .path(&args.block_path)?
                .build()
                .await?;

            let mut format_options = HashMap::new();
            if args.erase {
                format_options.insert("erase", zbus::zvariant::Value::from("zero"));
            }
            if let Some(label) = args.label.as_deref()
                && !label.is_empty()
            {
                format_options.insert("label", zbus::zvariant::Value::from(label));
            }

            proxy
                .format(args.filesystem_type.as_str(), format_options)
                .await?;
            Ok(())
        })
    }
}

fn common_partition_info_for(
    table_type: &str,
    selected_partition_type: usize,
) -> Result<&'static PartitionTypeInfo> {
    match table_type {
        "gpt" => COMMON_GPT_TYPES
            .get(selected_partition_type)
            .ok_or_else(|| anyhow::anyhow!("Invalid partition type index for GPT")),
        "dos" => COMMON_DOS_TYPES
            .get(selected_partition_type)
            .ok_or_else(|| anyhow::anyhow!("Invalid partition type index for DOS/MBR")),
        _ => Err(anyhow::anyhow!(
            "Unsupported partition table type: {table_type}"
        )),
    }
}

pub(crate) fn build_create_partition_and_format_args(
    table_block_path: String,
    table_type: &str,
    gpt_usable_range: Option<ByteRange>,
    info: CreatePartitionInfo,
) -> Result<CreatePartitionAndFormatArgs> {
    // UDisks2 expects bytes. When the user requests the maximum size for a free-space segment,
    // pass 0 to let the backend pick the maximal size after alignment/geometry constraints.
    let requested_size = if info.size >= info.max_size {
        0
    } else {
        info.size
    };

    // DOS/MBR typically reserves the beginning of the disk (MBR + alignment). Avoid targeting
    // offset 0.
    const DOS_RESERVED_START_BYTES: u64 = 1024 * 1024;
    if table_type == "dos" && info.offset < DOS_RESERVED_START_BYTES {
        return Err(anyhow::anyhow!(
            "Requested offset {} is inside reserved DOS/MBR start region (< {} bytes)",
            info.offset,
            DOS_RESERVED_START_BYTES
        ));
    }

    if table_type == "gpt"
        && let Some(range) = gpt_usable_range
    {
        if info.offset < range.start || info.offset >= range.end {
            return Err(anyhow::anyhow!(
                "Requested partition offset {} is outside GPT usable range [{}, {})",
                info.offset,
                range.start,
                range.end
            ));
        }

        if requested_size != 0 {
            let requested_end = info.offset.saturating_add(requested_size);
            if requested_end > range.end {
                return Err(anyhow::anyhow!(
                    "Requested partition range [{}, {}) is outside GPT usable range [{}, {})",
                    info.offset,
                    requested_end,
                    range.start,
                    range.end
                ));
            }
        }
    }

    // Find a partition type that matches the table type.
    // Note: UDisks2 reports DOS/MBR partition tables as "dos".
    let partition_info = common_partition_info_for(table_type, info.selected_partitition_type)?;

    if partition_info.table_type != table_type {
        return Err(anyhow::anyhow!(
            "Partition type '{}' is not compatible with partition table type '{}'",
            partition_info.name,
            table_type
        ));
    }

    let create_name = if table_type == "dos" {
        ""
    } else {
        info.name.as_str()
    };

    let create_partition_kind = if table_type == "dos" {
        Some("primary".to_string())
    } else {
        None
    };

    let label = if info.name.is_empty() {
        None
    } else {
        Some(info.name.clone())
    };

    Ok(CreatePartitionAndFormatArgs {
        table_block_path,
        table_type: table_type.to_string(),
        offset: info.offset,
        size: requested_size,
        partition_type: partition_info.ty.to_string(),
        create_name: create_name.to_string(),
        create_partition_kind,
        filesystem_type: partition_info.filesystem_type.to_string(),
        erase: info.erase,
        label,
    })
}

pub(crate) async fn drive_create_partition<B: DiskBackend>(
    backend: &B,
    table_block_path: String,
    table_type: &str,
    gpt_usable_range: Option<ByteRange>,
    info: CreatePartitionInfo,
) -> Result<()> {
    let args = build_create_partition_and_format_args(
        table_block_path,
        table_type,
        gpt_usable_range,
        info,
    )?;

    if let Err(e) = backend.create_partition_and_format(args.clone()).await {
        let fs = args.filesystem_type.as_str();
        let hint = match fs {
            "ntfs" => {
                " Hint: NTFS formatting requires mkfs.ntfs (usually provided by the 'ntfs-3g' package)."
            }
            "exfat" => {
                " Hint: exFAT formatting requires mkfs.exfat (usually provided by the 'exfatprogs' package)."
            }
            _ => "",
        };

        return Err(anyhow::anyhow!(
            "UDisks2 CreatePartitionAndFormat failed (table_type={}, offset={}, size={}, part_type={}, fs={}): {}.{}",
            args.table_type,
            args.offset,
            args.size,
            args.partition_type,
            fs,
            e,
            hint
        ));
    }

    Ok(())
}

pub(crate) async fn partition_mount<B: DiskBackend>(
    backend: &B,
    path: OwnedObjectPath,
) -> Result<()> {
    backend.fs_mount(path).await
}

pub(crate) async fn partition_unmount<B: DiskBackend>(
    backend: &B,
    path: OwnedObjectPath,
) -> Result<()> {
    backend.fs_unmount(path).await
}

pub(crate) async fn partition_delete<B: DiskBackend>(
    backend: &B,
    path: OwnedObjectPath,
) -> Result<()> {
    backend.partition_delete(path).await
}

pub(crate) async fn partition_format<B: DiskBackend>(
    backend: &B,
    args: PartitionFormatArgs,
) -> Result<()> {
    backend.block_format(args).await
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Call {
        Create(CreatePartitionAndFormatArgs),
        Mount(OwnedObjectPath),
        Unmount(OwnedObjectPath),
        Delete(OwnedObjectPath),
        Format(PartitionFormatArgs),
    }

    #[derive(Clone)]
    struct FakeBackend {
        calls: Arc<Mutex<Vec<Call>>>,
        create_result: Arc<Mutex<Result<(), String>>>,
        mount_result: Arc<Mutex<Result<(), String>>>,
        unmount_result: Arc<Mutex<Result<(), String>>>,
        delete_result: Arc<Mutex<Result<(), String>>>,
        format_result: Arc<Mutex<Result<(), String>>>,
    }

    impl Default for FakeBackend {
        fn default() -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
                create_result: Arc::new(Mutex::new(Ok(()))),
                mount_result: Arc::new(Mutex::new(Ok(()))),
                unmount_result: Arc::new(Mutex::new(Ok(()))),
                delete_result: Arc::new(Mutex::new(Ok(()))),
                format_result: Arc::new(Mutex::new(Ok(()))),
            }
        }
    }

    impl FakeBackend {
        fn set_create_result(&self, res: Result<()>) {
            *self.create_result.lock().unwrap() = res.map_err(|e| e.to_string());
        }

        fn set_mount_result(&self, res: Result<()>) {
            *self.mount_result.lock().unwrap() = res.map_err(|e| e.to_string());
        }

        fn set_unmount_result(&self, res: Result<()>) {
            *self.unmount_result.lock().unwrap() = res.map_err(|e| e.to_string());
        }

        fn set_delete_result(&self, res: Result<()>) {
            *self.delete_result.lock().unwrap() = res.map_err(|e| e.to_string());
        }

        fn set_format_result(&self, res: Result<()>) {
            *self.format_result.lock().unwrap() = res.map_err(|e| e.to_string());
        }

        fn take_calls(&self) -> Vec<Call> {
            std::mem::take(&mut *self.calls.lock().unwrap())
        }
    }

    impl DiskBackend for FakeBackend {
        fn create_partition_and_format(
            &self,
            args: CreatePartitionAndFormatArgs,
        ) -> BoxFuture<'_, Result<()>> {
            self.calls.lock().unwrap().push(Call::Create(args));
            let res = self.create_result.lock().unwrap().clone();
            Box::pin(async move { res.map_err(|e| anyhow::anyhow!(e)) })
        }

        fn fs_mount(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
            self.calls.lock().unwrap().push(Call::Mount(path));
            let res = self.mount_result.lock().unwrap().clone();
            Box::pin(async move { res.map_err(|e| anyhow::anyhow!(e)) })
        }

        fn fs_unmount(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
            self.calls.lock().unwrap().push(Call::Unmount(path));
            let res = self.unmount_result.lock().unwrap().clone();
            Box::pin(async move { res.map_err(|e| anyhow::anyhow!(e)) })
        }

        fn partition_delete(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
            self.calls.lock().unwrap().push(Call::Delete(path));
            let res = self.delete_result.lock().unwrap().clone();
            Box::pin(async move { res.map_err(|e| anyhow::anyhow!(e)) })
        }

        fn block_format(&self, args: PartitionFormatArgs) -> BoxFuture<'_, Result<()>> {
            self.calls.lock().unwrap().push(Call::Format(args));
            let res = self.format_result.lock().unwrap().clone();
            Box::pin(async move { res.map_err(|e| anyhow::anyhow!(e)) })
        }
    }

    #[test]
    fn build_args_gpt_uses_name_and_ext4_default() {
        let info = CreatePartitionInfo {
            name: "MyData".to_string(),
            offset: 2 * 1024 * 1024,
            size: 10,
            max_size: 100,
            erase: false,
            selected_partitition_type: 1, // Linux Filesystem (ext4)
            ..Default::default()
        };

        let args = build_create_partition_and_format_args(
            "/org/freedesktop/UDisks2/block_devices/sda".to_string(),
            "gpt",
            Some(ByteRange {
                start: 2 * 1024 * 1024,
                end: 100 * 1024 * 1024,
            }),
            info,
        )
        .expect("args should build");

        assert_eq!(args.create_name, "MyData");
        assert_eq!(args.filesystem_type, "ext4");
        assert_eq!(args.create_partition_kind, None);
    }

    #[tokio::test]
    async fn create_partition_failure_surfaces_error_and_hint_for_ntfs() {
        let backend = FakeBackend::default();
        backend.set_create_result(Err(anyhow::anyhow!("boom")));

        let info = CreatePartitionInfo {
            name: "Win".to_string(),
            offset: 2 * 1024 * 1024,
            size: 10,
            max_size: 100,
            erase: false,
            selected_partitition_type: 5, // Microsoft Basic Data (NTFS)
            ..Default::default()
        };

        let err = drive_create_partition(
            &backend,
            "/org/freedesktop/UDisks2/block_devices/sda".to_string(),
            "gpt",
            Some(ByteRange {
                start: 2 * 1024 * 1024,
                end: 100 * 1024 * 1024,
            }),
            info,
        )
        .await
        .unwrap_err();

        let msg = err.to_string();
        assert!(msg.contains("CreatePartitionAndFormat failed"));
        assert!(msg.contains("mkfs.ntfs"));

        let calls = backend.take_calls();
        assert!(matches!(calls.as_slice(), [Call::Create(_)]));
    }

    #[tokio::test]
    async fn mount_unmount_delete_format_calls_backend_and_propagates_errors() {
        let backend = FakeBackend::default();
        backend.set_mount_result(Err(anyhow::anyhow!("mount failed")));
        backend.set_unmount_result(Ok(()));
        backend.set_delete_result(Err(anyhow::anyhow!("delete failed")));
        backend.set_format_result(Err(anyhow::anyhow!("format failed")));

        let p: OwnedObjectPath = "/org/freedesktop/UDisks2/block_devices/sda1"
            .try_into()
            .unwrap();

        let mount_err = partition_mount(&backend, p.clone()).await.unwrap_err();
        assert!(mount_err.to_string().contains("mount failed"));

        partition_unmount(&backend, p.clone()).await.unwrap();

        let del_err = partition_delete(&backend, p.clone()).await.unwrap_err();
        assert!(del_err.to_string().contains("delete failed"));

        let fmt_err = partition_format(
            &backend,
            PartitionFormatArgs {
                block_path: p.clone(),
                filesystem_type: "ext4".to_string(),
                erase: true,
                label: Some("Data".to_string()),
            },
        )
        .await
        .unwrap_err();
        assert!(fmt_err.to_string().contains("format failed"));

        let calls = backend.take_calls();
        assert_eq!(calls.len(), 4);
    }
}
