// network_utils.rs
// This module provides utility functions to retrieve network device statistics
// and map device strings to friendly names on a Unix-based system.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use indexmap::IndexMap;

use crate::test_debug;

/// Retrieves network device statistics from the `/proc/net/dev` file.
///
/// # Returns
///
/// A result containing an IndexMap where the keys are the device names and the values are tuples of (received bytes, transmitted bytes).
/// In case of an error, returns an io::Error.
pub fn get_net_dev_stats() -> Result<IndexMap<String, (u64, u64)>, std::io::Error> {
    // Open the `/proc/net/dev` file for reading
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    // Parse the network device statistics from the file
    parse_net_dev_stats(reader)
}

/// Parses network device statistics from a given reader.
///
/// # Arguments
///
/// * `reader` - A reader that provides lines of network device statistics.
///
/// # Returns
///
/// A result containing an IndexMap where the keys are the device names and the values are tuples of (received bytes, transmitted bytes).
/// In case of an error, returns an io::Error.
pub fn parse_net_dev_stats<R: BufRead>(
    reader: R,
) -> Result<IndexMap<String, (u64, u64)>, std::io::Error> {
    let mut stats = IndexMap::new();
    let lines: Vec<_> = reader.lines().collect::<Result<_, _>>()?;
    test_debug!("Parsing {} lines", lines.len());

    // Skip the first two lines as they are headers
    for (_index, line) in lines.into_iter().enumerate().skip(2) {
        test_debug!("Parsing line: {}", line);
        // Split the line into interface name and the rest of the statistics
        if let Some((iface, rest)) = line.split_once(':') {
            let fields: Vec<&str> = rest.split_whitespace().collect();
            if fields.len() >= 9 {
                // Parse the received and transmitted bytes
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

/// Retrieves a map of device strings to friendly names.
///
/// # Returns
///
/// A HashMap where the keys are device strings and the values are friendly names.
pub fn get_device_string_to_name_map() -> HashMap<String, String> {
    HashMap::new() // This isn't really crucial on linux but we *could* implement it.
}
