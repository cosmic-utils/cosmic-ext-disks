// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderIcon {
    pub(crate) primary: Option<&'static str>,
    pub(crate) fallback_symbolic: &'static str,
    pub(crate) monogram: Option<&'static str>,
}

impl ProviderIcon {
    pub(crate) fn preferred_name(self) -> &'static str {
        self.primary.unwrap_or(self.fallback_symbolic)
    }
}

pub(crate) fn resolve_provider_icon(provider: &str) -> ProviderIcon {
    let provider = provider.trim().to_ascii_lowercase();

    match provider.as_str() {
        "drive" => ProviderIcon {
            primary: Some("google-drive-symbolic"),
            fallback_symbolic: "folder-remote-symbolic",
            monogram: Some("GD"),
        },
        "dropbox" => ProviderIcon {
            primary: Some("dropbox-symbolic"),
            fallback_symbolic: "folder-remote-symbolic",
            monogram: Some("DB"),
        },
        "onedrive" => ProviderIcon {
            primary: Some("onedrive-symbolic"),
            fallback_symbolic: "folder-remote-symbolic",
            monogram: Some("OD"),
        },
        "s3" => ProviderIcon {
            primary: Some("network-server-symbolic"),
            fallback_symbolic: "network-server-symbolic",
            monogram: Some("S3"),
        },
        "b2" => ProviderIcon {
            primary: Some("folder-remote-symbolic"),
            fallback_symbolic: "network-server-symbolic",
            monogram: Some("B2"),
        },
        "protondrive" => ProviderIcon {
            primary: Some("folder-remote-symbolic"),
            fallback_symbolic: "network-server-symbolic",
            monogram: Some("PD"),
        },
        "smb" => ProviderIcon {
            primary: Some("network-workgroup-symbolic"),
            fallback_symbolic: "network-server-symbolic",
            monogram: Some("SMB"),
        },
        "sftp" | "ftp" | "webdav" => ProviderIcon {
            primary: Some("network-server-symbolic"),
            fallback_symbolic: "folder-remote-symbolic",
            monogram: None,
        },
        _ => ProviderIcon {
            primary: None,
            fallback_symbolic: "folder-remote-symbolic",
            monogram: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_provider_icon;

    #[test]
    fn known_provider_has_primary_with_fallback() {
        let icon = resolve_provider_icon("dropbox");
        assert!(icon.primary.is_some());
        assert_eq!(icon.fallback_symbolic, "folder-remote-symbolic");
    }

    #[test]
    fn unknown_provider_uses_generic_fallback() {
        let icon = resolve_provider_icon("unknown-provider");
        assert!(icon.primary.is_none());
        assert_eq!(icon.fallback_symbolic, "folder-remote-symbolic");
    }
}
