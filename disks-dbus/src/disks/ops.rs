use std::collections::HashMap;

use anyhow::Result;
use futures::future::BoxFuture;
use tokio::time::{Duration, sleep};
use udisks2::{
    block::BlockProxy, filesystem::FilesystemProxy, partitiontable::PartitionTableProxy,
};
use zbus::zvariant::Value;
use zbus::{Connection, Proxy, zvariant::OwnedObjectPath};

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
    fn crypto_unlock(
        &self,
        path: OwnedObjectPath,
        passphrase: String,
    ) -> BoxFuture<'_, Result<OwnedObjectPath>>;
    fn crypto_lock(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>>;
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

    fn crypto_unlock(
        &self,
        path: OwnedObjectPath,
        passphrase: String,
    ) -> BoxFuture<'_, Result<OwnedObjectPath>> {
        Box::pin(async move {
            let proxy = udisks2::encrypted::EncryptedProxy::builder(&self.connection)
                .path(&path)?
                .build()
                .await?;
            let cleartext = proxy.unlock(&passphrase, HashMap::new()).await?;
            Ok(cleartext)
        })
    }

    fn crypto_lock(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let proxy = udisks2::encrypted::EncryptedProxy::builder(&self.connection)
                .path(&path)?
                .build()
                .await?;
            proxy.lock(HashMap::new()).await?;
            Ok(())
        })
    }

    fn partition_delete(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // Gather some best-effort context for error messages.
            let decode_c_string_bytes = |bytes: &[u8]| -> String {
                let raw = bytes.split(|b| *b == 0).next().unwrap_or(bytes);
                String::from_utf8_lossy(raw).to_string()
            };

            let open_fds_for_device = |needle: &str| -> Option<Vec<String>> {
                #[cfg(target_os = "linux")]
                {
                    use std::path::PathBuf;

                    if needle.trim().is_empty() {
                        return None;
                    }

                    let mut hits = Vec::new();
                    let dir = PathBuf::from("/proc/self/fd");
                    let Ok(entries) = std::fs::read_dir(&dir) else {
                        return None;
                    };

                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Ok(target) = std::fs::read_link(&path) {
                            let target_str = target.to_string_lossy();
                            if target_str.contains(needle) {
                                hits.push(format!("{} -> {}", path.display(), target_str));
                            }
                        }
                        if hits.len() >= 8 {
                            break;
                        }
                    }

                    if hits.is_empty() { None } else { Some(hits) }
                }

                #[cfg(not(target_os = "linux"))]
                {
                    let _ = needle;
                    None
                }
            };

            let mut device_for_display: Option<String> = None;
            let mut read_only: Option<bool> = None;
            if let Ok(builder) = BlockProxy::builder(&self.connection).path(&path) {
                if let Ok(block_proxy) = builder.build().await {
                    if let Ok(preferred) = block_proxy.preferred_device().await {
                        let s = decode_c_string_bytes(&preferred);
                        if !s.trim().is_empty() {
                            device_for_display = Some(s);
                        }
                    }
                    if device_for_display.is_none() {
                        if let Ok(dev) = block_proxy.device().await {
                            let s = decode_c_string_bytes(&dev);
                            if !s.trim().is_empty() {
                                device_for_display = Some(s);
                            }
                        }
                    }

                    read_only = block_proxy.read_only().await.ok();
                }
            }

            // Best-effort extra properties that `udisks2` crate doesn't currently expose.
            // These are extremely helpful to diagnose failures like EINVAL when the disk is in use.
            let mut hint_system: Option<bool> = None;
            let mut holders: Option<Vec<OwnedObjectPath>> = None;
            if let Ok(props) = Proxy::new(
                &self.connection,
                "org.freedesktop.UDisks2",
                &path,
                "org.freedesktop.DBus.Properties",
            )
            .await
            {
                hint_system = props
                    .call("Get", &("org.freedesktop.UDisks2.Block", "HintSystem"))
                    .await
                    .ok()
                    .and_then(|v: zbus::zvariant::OwnedValue| v.try_into().ok());

                holders = props
                    .call("Get", &("org.freedesktop.UDisks2.Block", "Holders"))
                    .await
                    .ok()
                    .and_then(|v: zbus::zvariant::OwnedValue| v.try_into().ok());
            }

            let mut partition_number: Option<u32> = None;
            let mut table_path: Option<OwnedObjectPath> = None;
            let mut table_type: Option<String> = None;
            let mut table_in_use_notes: Vec<String> = Vec::new();
            if let Ok(builder) =
                udisks2::partition::PartitionProxy::builder(&self.connection).path(&path)
            {
                if let Ok(partition_proxy) = builder.build().await {
                    partition_number = partition_proxy.number().await.ok();
                    table_path = partition_proxy.table().await.ok();

                    if let Some(tp) = table_path.as_ref() {
                        if let Ok(table_builder) =
                            PartitionTableProxy::builder(&self.connection).path(tp)
                        {
                            if let Ok(table_proxy) = table_builder.build().await {
                                table_type = table_proxy.type_().await.ok();

                                // Best-effort: check whether other partitions on the same disk are
                                // mounted or used as active swap. This often explains why modifying
                                // the partition table fails.
                                if let Ok(parts) = table_proxy.partitions().await {
                                    for part_path in parts {
                                        if part_path == path {
                                            continue;
                                        }

                                        // Device label for display.
                                        let mut part_dev: Option<String> = None;
                                        if let Ok(bb) =
                                            BlockProxy::builder(&self.connection).path(&part_path)
                                        {
                                            if let Ok(bp) = bb.build().await {
                                                if let Ok(preferred) = bp.preferred_device().await {
                                                    let s = decode_c_string_bytes(&preferred);
                                                    if !s.trim().is_empty() {
                                                        part_dev = Some(s);
                                                    }
                                                }
                                                if part_dev.is_none() {
                                                    if let Ok(dev) = bp.device().await {
                                                        let s = decode_c_string_bytes(&dev);
                                                        if !s.trim().is_empty() {
                                                            part_dev = Some(s);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Mounted filesystem?
                                        if let Ok(fs_builder) =
                                            FilesystemProxy::builder(&self.connection)
                                                .path(&part_path)
                                        {
                                            if let Ok(fs) = fs_builder.build().await {
                                                if let Ok(mps) = fs.mount_points().await {
                                                    let mps: Vec<String> = mps
                                                        .into_iter()
                                                        .filter_map(|mp| {
                                                            let s = decode_c_string_bytes(&mp);
                                                            if s.trim().is_empty() {
                                                                None
                                                            } else {
                                                                Some(s)
                                                            }
                                                        })
                                                        .collect();
                                                    if !mps.is_empty() {
                                                        let dev =
                                                            part_dev.clone().unwrap_or_else(|| {
                                                                part_path.to_string()
                                                            });
                                                        table_in_use_notes.push(format!(
                                                            "mounted: {dev} -> {}",
                                                            mps.join(", ")
                                                        ));
                                                    }
                                                }
                                            }
                                        }

                                        // Active swap?
                                        if let Ok(sw_builder) =
                                            udisks2::swapspace::SwapspaceProxy::builder(
                                                &self.connection,
                                            )
                                            .path(&part_path)
                                        {
                                            if let Ok(sw) = sw_builder.build().await {
                                                if let Ok(active) = sw.active().await {
                                                    if active {
                                                        let dev =
                                                            part_dev.clone().unwrap_or_else(|| {
                                                                part_path.to_string()
                                                            });
                                                        table_in_use_notes
                                                            .push(format!("active swap: {dev}"));
                                                    }
                                                }
                                            }
                                        }

                                        if table_in_use_notes.len() >= 8 {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // NOTE: We intentionally call this via a raw zbus proxy instead of the `udisks2`
            // crate proxy. `udisks2::error::Error` maps most MethodErrors to enum variants
            // and drops the original error message, leaving only “The operation failed”,
            // which is not actionable for users.

            // UDisks expects a{sv} (string -> variant). Use `Value` like the generated
            // `udisks2` proxies do.
            //
            // GNOME Disks calls `Partition.Delete` with *empty* options (`a{sv}`) after an
            // “ensure unused” preflight, so we do the same for maximum parity.
            let options_empty: HashMap<&str, Value<'_>> = HashMap::new();
            let mut options_teardown: HashMap<&str, Value<'_>> = HashMap::new();
            options_teardown.insert("tear-down", Value::from(true));

            let proxy = Proxy::new(
                &self.connection,
                "org.freedesktop.UDisks2",
                &path,
                "org.freedesktop.UDisks2.Partition",
            )
            .await?;

            // Keep the deletion attempt simple and GNOME Disks-like:
            // - try empty options first
            // - if the device is busy (udev/libblockdev races), retry briefly
            // - if that fails, try once with `tear-down=true` as a fallback

            let mut last_err: Option<zbus::Error> = None;
            for attempt in 0..4 {
                match proxy.call_method("Delete", &(options_empty)).await {
                    Ok(_) => return Ok(()),
                    Err(err) => {
                        let is_busy = matches!(&err,
                            zbus::Error::MethodError(name, _msg, _info)
                                if name.as_str() == "org.freedesktop.UDisks2.Error.DeviceBusy"
                        );

                        if is_busy && attempt < 3 {
                            last_err = Some(err);
                            sleep(Duration::from_millis(250)).await;
                            continue;
                        }

                        // Preserve D-Bus error name + message.
                        if let zbus::Error::MethodError(name, msg, _info) = &err {
                            let msg = msg.as_deref().unwrap_or("");

                            // Parity fallback: allow `tear-down=true` if empty options fail.
                            // Some stacks (esp. around crypto/DM) appear to need it.
                            if proxy
                                .call_method("Delete", &(options_teardown))
                                .await
                                .is_ok()
                            {
                                return Ok(());
                            }

                            let device =
                                device_for_display.as_deref().unwrap_or("<unknown device>");
                            let open_fds = open_fds_for_device(device)
                                .map(|v| v.join("; "))
                                .unwrap_or_else(|| "none".to_string());
                            let part_no = partition_number
                                .map(|n| n.to_string())
                                .unwrap_or_else(|| "?".to_string());
                            let table_path = table_path
                                .as_ref()
                                .map(|p| p.to_string())
                                .unwrap_or_else(|| "?".to_string());
                            let table_type = table_type.as_deref().unwrap_or("?");

                            let read_only = read_only
                                .map(|b| b.to_string())
                                .unwrap_or_else(|| "?".to_string());
                            let hint_system = hint_system
                                .map(|b| b.to_string())
                                .unwrap_or_else(|| "?".to_string());
                            let holders_str = holders
                                .as_ref()
                                .map(|hs| {
                                    hs.iter()
                                        .take(8)
                                        .map(|p| p.to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                })
                                .unwrap_or_else(|| "?".to_string());
                            let holders_count = holders.as_ref().map(|h| h.len()).unwrap_or(0);

                            let table_in_use = if table_in_use_notes.is_empty() {
                                "none".to_string()
                            } else {
                                table_in_use_notes.join("; ")
                            };
                            anyhow::bail!(
                                "UDisks2 Partition.Delete failed for {device} (partition_number={part_no}, table_type={table_type}, table_path={table_path}, object_path={path}, read_only={read_only}, hint_system={hint_system}, holders_count={holders_count}, holders={holders_str}, table_in_use={table_in_use}, open_fds_for_device={open_fds}): {}{}{}",
                                name.as_str(),
                                if msg.is_empty() { "" } else { ": " },
                                msg
                            );
                        }

                        return Err(err.into());
                    }
                }
            }

            if let Some(err) = last_err {
                return Err(err.into());
            }

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

pub(crate) async fn crypto_unlock(
    backend: &impl DiskBackend,
    path: OwnedObjectPath,
    passphrase: &str,
) -> Result<OwnedObjectPath> {
    // Never log passphrases.
    backend.crypto_unlock(path, passphrase.to_string()).await
}

pub(crate) async fn crypto_lock(backend: &impl DiskBackend, path: OwnedObjectPath) -> Result<()> {
    backend.crypto_lock(path).await
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
        CryptoUnlock(OwnedObjectPath),
        CryptoLock(OwnedObjectPath),
        Delete(OwnedObjectPath),
        Format(PartitionFormatArgs),
    }

    #[derive(Clone)]
    struct FakeBackend {
        calls: Arc<Mutex<Vec<Call>>>,
        create_result: Arc<Mutex<Result<(), String>>>,
        mount_result: Arc<Mutex<Result<(), String>>>,
        unmount_result: Arc<Mutex<Result<(), String>>>,
        crypto_unlock_result: Arc<Mutex<Result<OwnedObjectPath, String>>>,
        crypto_lock_result: Arc<Mutex<Result<(), String>>>,
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
                crypto_unlock_result: Arc::new(Mutex::new(Ok(
                    "/org/freedesktop/UDisks2/block_devices/dm_0"
                        .try_into()
                        .unwrap(),
                ))),
                crypto_lock_result: Arc::new(Mutex::new(Ok(()))),
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

        #[allow(dead_code)]
        fn set_crypto_unlock_result(&self, res: Result<OwnedObjectPath>) {
            *self.crypto_unlock_result.lock().unwrap() = res.map_err(|e| e.to_string());
        }

        #[allow(dead_code)]
        fn set_crypto_lock_result(&self, res: Result<()>) {
            *self.crypto_lock_result.lock().unwrap() = res.map_err(|e| e.to_string());
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

        fn crypto_unlock(
            &self,
            path: OwnedObjectPath,
            _passphrase: String,
        ) -> BoxFuture<'_, Result<OwnedObjectPath>> {
            self.calls.lock().unwrap().push(Call::CryptoUnlock(path));
            let res = self.crypto_unlock_result.lock().unwrap().clone();
            Box::pin(async move { res.map_err(|e| anyhow::anyhow!(e)) })
        }

        fn crypto_lock(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
            self.calls.lock().unwrap().push(Call::CryptoLock(path));
            let res = self.crypto_lock_result.lock().unwrap().clone();
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
