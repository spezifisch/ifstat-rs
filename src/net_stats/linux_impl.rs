use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};

use crate::test_debug;

pub fn get_net_dev_stats() -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    parse_net_dev_stats(reader)
}

pub fn parse_net_dev_stats<R: io::BufRead>(
    reader: R,
) -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let mut stats = HashMap::new();
    let lines: Vec<_> = reader.lines().collect::<Result<_, _>>()?;
    test_debug!("Parsing {} lines", lines.len());

    for (_index, line) in lines.into_iter().enumerate().skip(2) {
        test_debug!("Parsing line: {}", line);
        if let Some((iface, rest)) = line.split_once(':') {
            let fields: Vec<&str> = rest.split_whitespace().collect();
            if fields.len() >= 9 {
                let rx_bytes: u64 = fields[0].parse().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid RX bytes")
                })?;
                let tx_bytes: u64 = fields[8].parse().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid TX bytes")
                })?;
                stats.insert(iface.trim().to_string(), (rx_bytes, tx_bytes));
            } else {
                test_debug!(
                    "Invalid line format: '{}' ({} fields: {:?})",
                    line,
                    fields.len(),
                    fields
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Invalid line format: {} fields", fields.len()),
                ));
            }
        } else {
            test_debug!("Invalid line format: '{}' (no colon found)", line);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid line format (no colon found)",
            ));
        }
    }
    Ok(stats)
}

pub fn get_device_string_to_name_map() -> HashMap<String, String> {
    HashMap::new()
    // TODO
}
