use anyhow::Result;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub filesystem: String,
    pub blocks: u64,
    pub used: u64,
    pub available: u64,
    pub percent: u32,
    pub mount_point: String,
}

pub fn get_usage_data() -> Result<Vec<Usage>> {
    let output = Command::new("df").arg("--block-size=1").output()?;

    let text = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = text.lines().collect();

    let mut usages = vec![];
    for ln in 1..lines.len() {
        let values: Vec<&str> = lines[ln].split_whitespace().collect();

        if values.len() == 6 {
            usages.push(Usage {
                filesystem: values[0].to_string(),
                blocks: values[1].parse()?,
                used: values[2].parse()?,
                available: values[3].parse()?,
                percent: values[4].trim_end_matches('%').parse()?,
                mount_point: values[5].to_string(),
            });
        }
    }

    Ok(usages)
}
