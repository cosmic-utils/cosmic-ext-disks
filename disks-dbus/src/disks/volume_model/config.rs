use crate::udisks_block_config::ConfigurationItem;

/// Find a configuration item by type (e.g., "fstab" or "crypttab")
pub(super) fn find_configuration_item(
    items: &[ConfigurationItem],
    kind: &str,
) -> Option<ConfigurationItem> {
    items.iter().find(|(t, _)| t == kind).cloned()
}

/// Extract a value with a given prefix from a list of option tokens
pub(super) fn extract_prefixed_value(tokens: &[String], prefix: &str) -> String {
    tokens
        .iter()
        .find_map(|t| t.strip_prefix(prefix).map(|v| v.to_string()))
        .unwrap_or_default()
}
