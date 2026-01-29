use std::collections::HashMap;

use anyhow::Result;
use zbus::zvariant::{OwnedValue, Value};

use super::super::{SmartInfo, SmartSelfTestKind};
use super::is_anyhow_not_supported;
use super::model::DriveModel;

impl DriveModel {
    pub async fn smart_info(&self) -> Result<SmartInfo> {
        match self.nvme_smart_info().await {
            Ok(info) => Ok(info),
            Err(e) if is_anyhow_not_supported(&e) => match self.ata_smart_info().await {
                Ok(info) => Ok(info),
                Err(e2) if is_anyhow_not_supported(&e2) => {
                    Err(anyhow::anyhow!("Not supported by this drive"))
                }
                Err(e2) => Err(e2),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn smart_selftest_start(&self, kind: SmartSelfTestKind) -> Result<()> {
        match self.nvme_selftest_start(kind).await {
            Ok(()) => Ok(()),
            Err(e) if is_anyhow_not_supported(&e) => match self.ata_selftest_start(kind).await {
                Ok(()) => Ok(()),
                Err(e2) if is_anyhow_not_supported(&e2) => {
                    Err(anyhow::anyhow!("Not supported by this drive"))
                }
                Err(e2) => Err(e2),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn smart_selftest_abort(&self) -> Result<()> {
        match self.nvme_selftest_abort().await {
            Ok(()) => Ok(()),
            Err(e) if is_anyhow_not_supported(&e) => match self.ata_selftest_abort().await {
                Ok(()) => Ok(()),
                Err(e2) if is_anyhow_not_supported(&e2) => {
                    Err(anyhow::anyhow!("Not supported by this drive"))
                }
                Err(e2) => Err(e2),
            },
            Err(e) => Err(e),
        }
    }

    async fn nvme_smart_info(&self) -> Result<SmartInfo> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.NVMe.Controller",
        )
        .await?;

        // If the interface isn't present on this drive, properties/methods will error.
        let _state: String = proxy.get_property("State").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy.call("SmartUpdate", &(options)).await?;

        let updated_at: Option<u64> = proxy.get_property::<u64>("SmartUpdated").await.ok();
        let temp_k: Option<u16> = proxy.get_property::<u16>("SmartTemperature").await.ok();
        let power_on_hours: Option<u64> = proxy.get_property::<u64>("SmartPowerOnHours").await.ok();
        let selftest_status: Option<String> = proxy
            .get_property::<String>("SmartSelftestStatus")
            .await
            .ok();

        let attrs: HashMap<String, OwnedValue> = proxy
            .call("SmartGetAttributes", &(HashMap::<&str, Value<'_>>::new()))
            .await?;

        let mut attributes = std::collections::BTreeMap::new();
        for (k, v) in attrs {
            attributes.insert(k, format!("{v:?}"));
        }

        Ok(SmartInfo {
            device_type: "NVMe".to_string(),
            updated_at,
            temperature_c: temp_k.map(|k| (k as u64).saturating_sub(273)),
            power_on_hours,
            selftest_status,
            attributes,
        })
    }

    async fn nvme_selftest_start(&self, kind: SmartSelfTestKind) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.NVMe.Controller",
        )
        .await?;

        let _state: String = proxy.get_property("State").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy
            .call("SmartSelftestStart", &(kind.as_udisks_str(), options))
            .await?;
        Ok(())
    }

    async fn nvme_selftest_abort(&self) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.NVMe.Controller",
        )
        .await?;

        let _state: String = proxy.get_property("State").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy.call("SmartSelftestAbort", &(options)).await?;
        Ok(())
    }

    async fn ata_smart_info(&self) -> Result<SmartInfo> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.Drive.Ata",
        )
        .await?;

        // If the interface isn't present on this drive, this will error.
        let _smart_enabled: bool = proxy.get_property("SmartEnabled").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy.call("SmartUpdate", &(options)).await?;

        let updated_at: Option<u64> = proxy.get_property::<u64>("SmartUpdated").await.ok();
        let temperature: Option<u64> = proxy.get_property::<u64>("SmartTemperature").await.ok();
        let power_on_seconds: Option<u64> =
            proxy.get_property::<u64>("SmartPowerOnSeconds").await.ok();
        let selftest_status: Option<String> = proxy
            .get_property::<String>("SmartSelftestStatus")
            .await
            .ok();

        let attrs: HashMap<String, OwnedValue> = proxy
            .call("SmartGetAttributes", &(HashMap::<&str, Value<'_>>::new()))
            .await?;

        let mut attributes = std::collections::BTreeMap::new();
        for (k, v) in attrs {
            attributes.insert(k, format!("{v:?}"));
        }

        Ok(SmartInfo {
            device_type: "ATA".to_string(),
            updated_at,
            temperature_c: temperature,
            power_on_hours: power_on_seconds.map(|s| s / 3600),
            selftest_status,
            attributes,
        })
    }

    async fn ata_selftest_start(&self, kind: SmartSelfTestKind) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.Drive.Ata",
        )
        .await?;

        let _smart_enabled: bool = proxy.get_property("SmartEnabled").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy
            .call("SmartSelftestStart", &(kind.as_udisks_str(), options))
            .await?;
        Ok(())
    }

    async fn ata_selftest_abort(&self) -> Result<()> {
        let proxy = zbus::Proxy::new(
            &self.connection,
            "org.freedesktop.UDisks2",
            self.path.as_str(),
            "org.freedesktop.UDisks2.Drive.Ata",
        )
        .await?;

        let _smart_enabled: bool = proxy.get_property("SmartEnabled").await?;

        let options: HashMap<&str, Value<'_>> = HashMap::new();
        let _: () = proxy.call("SmartSelftestAbort", &(options)).await?;
        Ok(())
    }
}
