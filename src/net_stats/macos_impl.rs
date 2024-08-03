use indexmap::IndexMap;
use std::io::{self, BufReader};
use std::process::Command;

pub fn get_net_dev_stats() -> Result<IndexMap<String, (u64, u64)>, std::io::Error> {
    let output = Command::new("netstat")
        .arg("-i")
        .output()
        .expect("Failed to execute netstat command");
    let reader = BufReader::new(output.stdout.as_slice());
    parse_net_dev_stats(reader)
}

pub fn parse_net_dev_stats<R: io::BufRead>(
    reader: R,
) -> Result<IndexMap<String, (u64, u64)>, std::io::Error> {
    let mut stats = IndexMap::new();
    let lines: Vec<_> = reader.lines().collect::<Result<_, _>>()?;
    for line in lines.iter().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 10 {
            let iface = fields[0].to_string();
            let rx_bytes: u64 = fields[6].parse().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid RX bytes")
            })?;
            let tx_bytes: u64 = fields[9].parse().map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid TX bytes")
            })?;
            stats.insert(iface, (rx_bytes, tx_bytes));
        }
    }
    Ok(stats)
}

pub fn get_device_string_to_name_map() -> IndexMap<String, String> {
    IndexMap::new()
    // TODO
}
