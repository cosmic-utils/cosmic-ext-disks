mod actions;
mod discovery;
mod model;
mod smart;
mod volume_tree;

pub use model::DriveModel;

pub(super) fn is_dbus_not_supported(err: &zbus::Error) -> bool {
    match err {
        zbus::Error::MethodError(name, _msg, _info) => matches!(
            name.as_str(),
            "org.freedesktop.DBus.Error.UnknownInterface"
                | "org.freedesktop.DBus.Error.UnknownMethod"
                | "org.freedesktop.DBus.Error.UnknownProperty"
        ),
        _ => false,
    }
}

pub(super) fn is_dbus_device_busy(err: &zbus::Error) -> bool {
    match err {
        zbus::Error::MethodError(name, _msg, _info) => {
            name.as_str() == "org.freedesktop.UDisks2.Error.DeviceBusy"
        }
        _ => false,
    }
}

pub(super) fn is_anyhow_not_supported(err: &anyhow::Error) -> bool {
    err.downcast_ref::<zbus::Error>()
        .is_some_and(is_dbus_not_supported)
}

pub(super) fn is_anyhow_device_busy(err: &anyhow::Error) -> bool {
    err.downcast_ref::<zbus::Error>()
        .is_some_and(is_dbus_device_busy)
}

#[cfg(test)]
mod tests {
    #[test]
    fn dos_table_type_is_supported_and_not_msdos() {
        assert!(crate::COMMON_DOS_TYPES[0].table_type == "dos");
    }

    #[test]
    fn gpt_table_type_is_supported() {
        assert!(crate::COMMON_GPT_TYPES[0].table_type == "gpt");
    }
}
