// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use storage_sys::usage::{scan_local_mounts, scan_paths, ScanConfig};

#[derive(Debug, Parser)]
#[command(name = "scan-categories")]
#[command(about = "Scan file categories and byte totals quickly on Linux")]
struct Args {
    #[arg(long, default_value = "/")]
    root: PathBuf,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    threads: Option<usize>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = ScanConfig {
        threads: args.threads,
    };

    let result = if args.root == PathBuf::from("/") {
        scan_local_mounts(&args.root, &config)?
    } else {
        scan_paths(std::slice::from_ref(&args.root), &config)?
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    println!("CATEGORY       BYTES            PERCENT");
    println!("----------------------------------------");

    for entry in &result.categories {
        let percent = if result.total_bytes == 0 {
            0.0
        } else {
            (entry.bytes as f64 * 100.0) / result.total_bytes as f64
        };

        println!(
            "{:<13} {:>14} {:>9.2}%",
            entry.category.as_str(),
            entry.bytes,
            percent
        );
    }

    println!();
    println!(
        "total_bytes={} files_scanned={} dirs_scanned={} skipped_errors={} mounts_scanned={} elapsed_ms={}",
        result.total_bytes,
        result.files_scanned,
        result.dirs_scanned,
        result.skipped_errors,
        result.mounts_scanned,
        result.elapsed_ms
    );

    Ok(())
}
