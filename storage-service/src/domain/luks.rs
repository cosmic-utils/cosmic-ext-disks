// SPDX-License-Identifier: GPL-3.0-only

pub trait LuksDomain: Send + Sync {
    fn normalize_luks_version<'a>(&self, version: &'a str) -> zbus::fdo::Result<&'a str>;
}

pub struct DefaultLuksDomain;

impl LuksDomain for DefaultLuksDomain {
    fn normalize_luks_version<'a>(&self, version: &'a str) -> zbus::fdo::Result<&'a str> {
        if version.is_empty() || version == "luks2" {
            Ok("luks2")
        } else if version == "luks1" {
            Ok("luks1")
        } else {
            Err(zbus::fdo::Error::InvalidArgs(format!(
                "Invalid LUKS version: {}. Use 'luks1' or 'luks2'",
                version
            )))
        }
    }
}
