use std::{collections::HashSet, io, path::Path, process::Command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LvmLogicalVolumeInfo {
    pub vg_name: String,
    pub lv_path: String,
    pub size_bytes: u64,
}

impl LvmLogicalVolumeInfo {
    pub fn display_name(&self) -> String {
        // Prefer a short human label; fall back to the LV path.
        // /dev/vg/lv -> vg/lv
        if let Some(stripped) = self.lv_path.strip_prefix("/dev/") {
            return stripped.to_string();
        }
        self.lv_path.clone()
    }
}

fn canonicalize_best_effort(p: &str) -> String {
    match std::fs::canonicalize(p) {
        Ok(c) => c.to_string_lossy().to_string(),
        Err(_) => p.to_string(),
    }
}

fn parse_pvs_vg_names(output: &str) -> Vec<(String, String)> {
    // Expected lines like: "vg0\t/dev/dm-2" (no headings).
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let mut parts = line.split('\t');
            let vg = parts.next()?.trim().to_string();
            let pv = parts.next()?.trim().to_string();
            if vg.is_empty() || pv.is_empty() {
                None
            } else {
                Some((vg, pv))
            }
        })
        .collect()
}

fn parse_lvs(output: &str, vg_name: &str) -> Vec<LvmLogicalVolumeInfo> {
    // Expected lines like: "/dev/vg0/root\t10737418240" (bytes, no suffix).
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let mut parts = line.split('\t');
            let lv_path = parts.next()?.trim().to_string();
            let size_str = parts.next()?.trim();
            let size_bytes: u64 = size_str.parse().ok()?;
            if lv_path.is_empty() {
                None
            } else {
                Some(LvmLogicalVolumeInfo {
                    vg_name: vg_name.to_string(),
                    lv_path,
                    size_bytes,
                })
            }
        })
        .collect()
}

pub fn list_lvs_for_pv(pv_device: &str) -> io::Result<Vec<LvmLogicalVolumeInfo>> {
    // LVM support is required by spec; we still degrade gracefully if tools are missing.
    if !Path::new("/sbin/pvs").exists() && which_in_path("pvs").is_none() {
        return Ok(Vec::new());
    }
    if !Path::new("/sbin/lvs").exists() && which_in_path("lvs").is_none() {
        return Ok(Vec::new());
    }

    let pv_canon = canonicalize_best_effort(pv_device);

    let pvs_out = Command::new("pvs")
        .args([
            "--noheadings",
            "--units",
            "b",
            "--nosuffix",
            "-o",
            "vg_name,pv_name",
            "--separator",
            "\t",
        ])
        .output()?;

    let pvs_text = String::from_utf8_lossy(&pvs_out.stdout).to_string();
    let mappings = parse_pvs_vg_names(&pvs_text);

    let mut vgs = HashSet::new();
    for (vg, pv) in mappings {
        let pv_match = canonicalize_best_effort(&pv);
        if pv_match == pv_canon {
            vgs.insert(vg);
        }
    }

    let mut all_lvs = Vec::new();
    for vg in vgs {
        let lvs_out = Command::new("lvs")
            .args([
                "--noheadings",
                "--units",
                "b",
                "--nosuffix",
                "-o",
                "lv_path,lv_size",
                "--separator",
                "\t",
                vg.as_str(),
            ])
            .output()?;

        let lvs_text = String::from_utf8_lossy(&lvs_out.stdout).to_string();
        all_lvs.extend(parse_lvs(&lvs_text, &vg));
    }

    Ok(all_lvs)
}

fn which_in_path(cmd: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(cmd);
        if candidate.exists() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pvs_mapping() {
        let out = "vg0\t/dev/dm-2\nvg1\t/dev/sda3\n";
        let v = parse_pvs_vg_names(out);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].0, "vg0");
        assert_eq!(v[0].1, "/dev/dm-2");
    }

    #[test]
    fn parses_lvs_lines() {
        let out = "/dev/vg0/root\t10737418240\n/dev/vg0/home\t2147483648\n";
        let v = parse_lvs(out, "vg0");
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].lv_path, "/dev/vg0/root");
        assert_eq!(v[0].size_bytes, 10737418240);
        assert_eq!(v[0].vg_name, "vg0");
    }
}
