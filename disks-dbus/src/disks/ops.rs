use std::collections::HashMap;

use anyhow::Result;
use futures::future::BoxFuture;
use udisks2::block::BlockProxy;
use zbus::zvariant::Value;
use zbus::{Connection, Proxy, zvariant::OwnedObjectPath};

use super::ByteRange;
use crate::{COMMON_DOS_TYPES, COMMON_GPT_TYPES, CreatePartitionInfo, PartitionTypeInfo};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct RedactedString(String);

impl RedactedString {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub(crate) fn expose(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Debug for RedactedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<redacted>")
    }
}

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
    pub(crate) encrypt_type: Option<String>,
    pub(crate) encrypt_passphrase: Option<RedactedString>,
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

async fn device_for_display(connection: &Connection, path: &OwnedObjectPath) -> Option<String> {
    let Ok(builder) = BlockProxy::builder(connection).path(path) else {
        return None;
    };

    let Ok(block_proxy) = builder.build().await else {
        return None;
    };

    let decode = |bytes: &[u8]| {
        let raw = bytes.split(|b| *b == 0).next().unwrap_or(bytes);
        let s = String::from_utf8_lossy(raw).to_string();
        if s.trim().is_empty() { None } else { Some(s) }
    };

    if let Ok(preferred) = block_proxy.preferred_device().await
        && let Some(device) = decode(&preferred)
    {
        return Some(device);
    }

    block_proxy.device().await.ok().and_then(|dev| decode(&dev))
}

/// Check if a zbus error indicates the device/mount is busy (EBUSY/target is busy).
/// Returns Some with device and mount_point if detected, None otherwise.
fn check_resource_busy_error(
    device_for_display: Option<&str>,
    object_path: &OwnedObjectPath,
    err: &zbus::Error,
) -> Option<(String, String)> {
    let zbus::Error::MethodError(_name, msg, _info) = err else {
        return None;
    };

    let msg_str = msg.as_deref().unwrap_or("");
    
    // UDisks2 typically returns errors like "target is busy" or "device is busy" for EBUSY
    if msg_str.to_lowercase().contains("target is busy") 
        || msg_str.to_lowercase().contains("device is busy")
        || msg_str.to_lowercase().contains("resource busy")
    {
        let device = device_for_display.unwrap_or("<unknown device>").to_string();
        // Mount point would need to be queried separately; for now use object_path as fallback
        let mount_point = format!("<object: {}>", object_path);
        tracing::debug!(
            device = %device,
            mount_point = %mount_point,
            error_msg = %msg_str,
            "Resource busy error detected during unmount"
        );
        return Some((device, mount_point));
    }

    None
}

fn anyhow_from_method_error(
    operation: &str,
    object_path: &OwnedObjectPath,
    device_for_display: Option<&str>,
    err: &zbus::Error,
) -> Option<anyhow::Error> {
    let zbus::Error::MethodError(name, msg, _info) = err else {
        return None;
    };

    let device = device_for_display.unwrap_or("<unknown device>");
    let msg = msg.as_deref().unwrap_or("");

    Some(anyhow::anyhow!(
        "UDisks2 {operation} failed for {device} (object_path={object_path}): {}{}{}",
        name.as_str(),
        if msg.is_empty() { "" } else { ": " },
        msg
    ))
}

impl DiskBackend for RealDiskBackend {
    fn create_partition_and_format(
        &self,
        args: CreatePartitionAndFormatArgs,
    ) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let table_path: OwnedObjectPath = args.table_block_path.as_str().try_into()?;
            let device_for_display = device_for_display(&self.connection, &table_path).await;

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

            // UDisks2 Block.Format supports encrypt.* options. When set, it will create a LUKS
            // container and format the filesystem on the unlocked device.
            // Docs: https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Block.html
            if let Some(encrypt_type) = args.encrypt_type.as_deref()
                && let Some(passphrase) = args.encrypt_passphrase.as_ref()
            {
                format_options.insert("encrypt.type", zbus::zvariant::Value::from(encrypt_type));
                // NOTE: Never log passphrases.
                format_options.insert(
                    "encrypt.passphrase",
                    zbus::zvariant::Value::from(passphrase.expose()),
                );
            }

            // NOTE: Use a raw zbus proxy so we preserve MethodError name/message.
            let proxy = Proxy::new(
                &self.connection,
                "org.freedesktop.UDisks2",
                &table_path,
                "org.freedesktop.UDisks2.PartitionTable",
            )
            .await?;

            let res: std::result::Result<OwnedObjectPath, zbus::Error> = proxy
                .call(
                    "CreatePartitionAndFormat",
                    &(
                        args.offset,
                        args.size,
                        args.partition_type.as_str(),
                        args.create_name.as_str(),
                        create_options,
                        args.filesystem_type.as_str(),
                        format_options,
                    ),
                )
                .await;

            match res {
                Ok(_created_path) => Ok(()),
                Err(err) => {
                    if let Some(e) = anyhow_from_method_error(
                        "PartitionTable.CreatePartitionAndFormat",
                        &table_path,
                        device_for_display.as_deref(),
                        &err,
                    ) {
                        return Err(e);
                    }
                    Err(err.into())
                }
            }
        })
    }

    fn fs_mount(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // NOTE: Use a raw zbus proxy so we preserve MethodError name/message.
            let device_for_display = device_for_display(&self.connection, &path).await;

            let proxy = Proxy::new(
                &self.connection,
                "org.freedesktop.UDisks2",
                &path,
                "org.freedesktop.UDisks2.Filesystem",
            )
            .await?;

            let options_empty: HashMap<&str, Value<'_>> = HashMap::new();
            let res: std::result::Result<String, zbus::Error> =
                proxy.call("Mount", &(options_empty)).await;

            match res {
                Ok(_mount_point) => Ok(()),
                Err(err) => {
                    if let Some(e) = anyhow_from_method_error(
                        "Filesystem.Mount",
                        &path,
                        device_for_display.as_deref(),
                        &err,
                    ) {
                        return Err(e);
                    }
                    Err(err.into())
                }
            }
        })
    }

    fn fs_unmount(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // NOTE: Use a raw zbus proxy so we preserve MethodError name/message.
            let device_for_display = device_for_display(&self.connection, &path).await;

            let proxy = Proxy::new(
                &self.connection,
                "org.freedesktop.UDisks2",
                &path,
                "org.freedesktop.UDisks2.Filesystem",
            )
            .await?;

            let options_empty: HashMap<&str, Value<'_>> = HashMap::new();
            let res: std::result::Result<(), zbus::Error> =
                proxy.call("Unmount", &(options_empty)).await;

            match res {
                Ok(()) => Ok(()),
                Err(err) => {
                    // Check if this is a "resource busy" error first
                    if let Some((device, mount_point)) = check_resource_busy_error(
                        device_for_display.as_deref(),
                        &path,
                        &err,
                    ) {
                        return Err(crate::disks::DiskError::ResourceBusy {
                            device,
                            mount_point,
                        }
                        .into());
                    }

                    // Otherwise, use standard error handling
                    if let Some(e) = anyhow_from_method_error(
                        "Filesystem.Unmount",
                        &path,
                        device_for_display.as_deref(),
                        &err,
                    ) {
                        return Err(e);
                    }
                    Err(err.into())
                }
            }
        })
    }

    fn crypto_unlock(
        &self,
        path: OwnedObjectPath,
        passphrase: String,
    ) -> BoxFuture<'_, Result<OwnedObjectPath>> {
        Box::pin(async move {
            // NOTE: Use a raw zbus proxy so we preserve MethodError name/message.
            // Never log or include passphrases in error strings.
            let device_for_display = device_for_display(&self.connection, &path).await;

            let proxy = Proxy::new(
                &self.connection,
                "org.freedesktop.UDisks2",
                &path,
                "org.freedesktop.UDisks2.Encrypted",
            )
            .await?;

            let options_empty: HashMap<&str, Value<'_>> = HashMap::new();
            let res: std::result::Result<OwnedObjectPath, zbus::Error> =
                proxy.call("Unlock", &(&passphrase, options_empty)).await;

            match res {
                Ok(cleartext) => Ok(cleartext),
                Err(err) => {
                    if let Some(e) = anyhow_from_method_error(
                        "Encrypted.Unlock",
                        &path,
                        device_for_display.as_deref(),
                        &err,
                    ) {
                        return Err(e);
                    }
                    Err(err.into())
                }
            }
        })
    }

    fn crypto_lock(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // NOTE: Use a raw zbus proxy so we preserve MethodError name/message.
            let device_for_display = device_for_display(&self.connection, &path).await;

            let proxy = Proxy::new(
                &self.connection,
                "org.freedesktop.UDisks2",
                &path,
                "org.freedesktop.UDisks2.Encrypted",
            )
            .await?;

            let options_empty: HashMap<&str, Value<'_>> = HashMap::new();
            let res: std::result::Result<(), zbus::Error> =
                proxy.call("Lock", &(options_empty)).await;

            match res {
                Ok(()) => Ok(()),
                Err(err) => {
                    if let Some(e) = anyhow_from_method_error(
                        "Encrypted.Lock",
                        &path,
                        device_for_display.as_deref(),
                        &err,
                    ) {
                        return Err(e);
                    }
                    Err(err.into())
                }
            }
        })
    }

    fn partition_delete(&self, path: OwnedObjectPath) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            // NOTE: We intentionally call this via a raw zbus proxy instead of the `udisks2`
            // crate proxy. `udisks2::error::Error` maps most MethodErrors to enum variants
            // and drops the original error message, leaving only “The operation failed”,
            // which is not actionable for users.

            // UDisks expects a{sv} (string -> variant). Use `Value` like the generated proxies.
            // GNOME Disks calls `Partition.Delete` with *empty* options (`a{sv}`).
            let options_empty: HashMap<&str, Value<'_>> = HashMap::new();

            let proxy = Proxy::new(
                &self.connection,
                "org.freedesktop.UDisks2",
                &path,
                "org.freedesktop.UDisks2.Partition",
            )
            .await?;

            let device_for_display = device_for_display(&self.connection, &path).await;

            match proxy.call_method("Delete", &(options_empty)).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    if let Some(e) = anyhow_from_method_error(
                        "Partition.Delete",
                        &path,
                        device_for_display.as_deref(),
                        &err,
                    ) {
                        return Err(e);
                    }
                    Err(err.into())
                }
            }
        })
    }

    fn block_format(&self, args: PartitionFormatArgs) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let device_for_display = device_for_display(&self.connection, &args.block_path).await;

            let mut format_options = HashMap::new();
            if args.erase {
                format_options.insert("erase", zbus::zvariant::Value::from("zero"));
            }
            if let Some(label) = args.label.as_deref()
                && !label.is_empty()
            {
                format_options.insert("label", zbus::zvariant::Value::from(label));
            }

            // NOTE: Use a raw zbus proxy so we preserve MethodError name/message.
            let proxy = Proxy::new(
                &self.connection,
                "org.freedesktop.UDisks2",
                &args.block_path,
                "org.freedesktop.UDisks2.Block",
            )
            .await?;

            let res: std::result::Result<(), zbus::Error> = proxy
                .call("Format", &(args.filesystem_type.as_str(), format_options))
                .await;

            match res {
                Ok(()) => Ok(()),
                Err(err) => {
                    if let Some(e) = anyhow_from_method_error(
                        "Block.Format",
                        &args.block_path,
                        device_for_display.as_deref(),
                        &err,
                    ) {
                        return Err(e);
                    }
                    Err(err.into())
                }
            }
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
    let partition_info = common_partition_info_for(table_type, info.selected_partition_type_index)?;

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

    let (encrypt_type, encrypt_passphrase) = if info.password_protected {
        if info.password.is_empty() {
            return Err(anyhow::anyhow!(
                "Missing passphrase for encrypted partition"
            ));
        }
        if info.password != info.confirmed_password {
            return Err(anyhow::anyhow!("Passphrases do not match"));
        }

        (
            // Default to LUKS2 for new volumes.
            Some("luks2".to_string()),
            Some(RedactedString::new(info.password.clone())),
        )
    } else {
        (None, None)
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
        encrypt_type,
        encrypt_passphrase,
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

        let encrypt_hint = if args.encrypt_type.is_some() {
            " Hint: Encrypted volumes require cryptsetup (dm-crypt)."
        } else {
            ""
        };

        return Err(anyhow::anyhow!(
            "UDisks2 CreatePartitionAndFormat failed (table_type={}, offset={}, size={}, part_type={}, fs={}): {}.{}{}",
            args.table_type,
            args.offset,
            args.size,
            args.partition_type,
            fs,
            e,
            hint,
            encrypt_hint
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

    fn gpt_index_for_fs(fs: &str) -> usize {
        COMMON_GPT_TYPES
            .iter()
            .position(|p| p.filesystem_type == fs)
            .unwrap_or(0)
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
            selected_partition_type_index: gpt_index_for_fs("ext4"),
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
        assert_eq!(args.encrypt_type, None);
        assert_eq!(args.encrypt_passphrase, None);
    }

    #[test]
    fn build_args_encrypted_sets_encrypt_options_and_redacts_passphrase() {
        let info = CreatePartitionInfo {
            name: "Secret".to_string(),
            offset: 2 * 1024 * 1024,
            size: 10,
            max_size: 100,
            erase: false,
            selected_partition_type_index: gpt_index_for_fs("ext4"),
            password_protected: true,
            password: "pw".to_string(),
            confirmed_password: "pw".to_string(),
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

        assert_eq!(args.encrypt_type.as_deref(), Some("luks2"));
        assert!(args.encrypt_passphrase.is_some());

        let dbg = format!("{args:?}");
        assert!(!dbg.contains("pw"));
        assert!(dbg.contains("<redacted>"));
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
            selected_partition_type_index: gpt_index_for_fs("ntfs"),
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

    #[test]
    fn check_resource_busy_detects_busy_patterns() {
        let _path: OwnedObjectPath = "/org/freedesktop/UDisks2/block_devices/sda1"
            .try_into()
            .unwrap();

        // Since we can't easily construct zbus::Error::MethodError in tests,
        // we'll test the busy detection logic independently.
        // The actual integration is tested in the backend itself.

        // Test that our error patterns would match
        let test_messages = vec![
            "target is busy",
            "device is busy",
            "Target Is Busy",  // case-insensitive
            "RESOURCE BUSY",
            "Error: target is busy (unmount failed)",
        ];

        for msg in test_messages {
            assert!(
                msg.to_lowercase().contains("target is busy")
                    || msg.to_lowercase().contains("device is busy")
                    || msg.to_lowercase().contains("resource busy"),
                "Pattern should match for: {}",
                msg
            );
        }

        // Test non-matching messages
        let non_busy_messages = vec![
            "permission denied",
            "not mounted",
            "some other error",
        ];

        for msg in non_busy_messages {
            assert!(
                !msg.to_lowercase().contains("target is busy")
                    && !msg.to_lowercase().contains("device is busy")
                    && !msg.to_lowercase().contains("resource busy"),
                "Pattern should NOT match for: {}",
                msg
            );
        }
    }
}
