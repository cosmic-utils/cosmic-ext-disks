use anyhow::Result;
use num_format::{Locale, ToFormattedString};
use std::{cmp::min, f64};

pub fn bytes_to_pretty(bytes: &u64, add_bytes: bool) -> String {
    let mut steps = 0;
    let mut val: f64 = bytes.clone() as f64;

    while val > 1024. && steps <= 8 {
        val = val / 1024.;
        steps += 1;
    }

    let unit = match steps {
        0 => "B",
        1 => "KB",
        2 => "MB",
        3 => "GB",
        4 => "TB",
        5 => "PB",
        6 => "EB",
        7 => "ZB",
        8 => "YB",
        _ => "Not Supported",
    };

    if add_bytes {
        let bytes_str = bytes.to_formatted_string(&Locale::en); //TODO: Accept locale as a parameter.
        return format!("{:.2} {} ({} bytes)", val, unit, bytes_str);
    } else {
        return format!("{:.2} {}", val, unit);
    }
}

//method to parse pretty bytes into numeric bytes
pub fn pretty_to_bytes(pretty: &str) -> Result<u64> {
    let mut steps = 0;

    let split = pretty.split_whitespace().collect::<Vec<&str>>();
    let string_value = split.first();

    if string_value.is_none() {
        return Err(anyhow::anyhow!("Invalid input"));
    }
    let string_value = string_value.unwrap();

    let mut val: f64 = string_value.parse()?;
    let unit = pretty.split_whitespace().last().unwrap();

    match unit {
        "B" => steps = 0,
        "KB" => steps = 1,
        "MB" => steps = 2,
        "GB" => steps = 3,
        "TB" => steps = 4,
        "PB" => steps = 5,
        "EB" => steps = 6,
        "ZB" => steps = 7,
        "YB" => steps = 8,
        _ => steps = 0,
    }

    while steps > 0 {
        val = val * 1024.;
        steps -= 1;
    }

    Ok(val as u64)
}

//method to return numeric value that would be displayed in bytes_to_pretty
pub fn get_numeric(bytes: &u64) -> f64 {
    let mut steps = 0;
    let mut val: f64 = bytes.clone() as f64;

    while val > 1024. && steps <= 8 {
        val = val / 1024.;
        steps += 1;
    }

    val
}

//method to return decent step value for numeric boxes based on displayed value, determined by bytes_to_pretty
pub fn get_step(bytes: &u64) -> f64 {
    let mut denomination = 0;
    let mut val: f64 = bytes.clone() as f64;

    while val > 1024. && denomination <= 8 {
        val = val / 1024.;
        denomination += 1;
    }

    return 1024_f64.powi(denomination);
}
