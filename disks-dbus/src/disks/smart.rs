use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartSelfTestKind {
    Short,
    Extended,
}

impl SmartSelfTestKind {
    pub(crate) fn as_udisks_str(self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::Extended => "extended",
        }
    }
}

/// A minimal, UI-friendly view of device SMART/health data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SmartInfo {
    /// Human-readable identifier for the backend interface providing SMART.
    /// Examples: "NVMe", "ATA".
    pub device_type: String,

    /// Seconds since epoch (UTC) when SMART data was last updated (if available).
    pub updated_at: Option<u64>,

    /// Temperature in Celsius (if available).
    pub temperature_c: Option<u64>,

    /// Power-on hours (if available).
    pub power_on_hours: Option<u64>,

    /// Self-test status (if available).
    pub selftest_status: Option<String>,

    /// Additional attributes (key → stringified value), ordered by key.
    pub attributes: BTreeMap<String, String>,
}
