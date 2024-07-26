use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    loop {
        match read_and_parse_net_dev() {
            Ok(stats) => {
                for (interface, (rx, tx)) in stats {
                    println!(
                        "Interface: {}, RX: {} bytes, TX: {} bytes",
                        interface, rx, tx
                    );
                }
            }
            Err(e) => eprintln!("Error reading /proc/net/dev: {}", e),
        }
        sleep(Duration::from_secs(1)).await;
    }
}

fn read_and_parse_net_dev() -> Result<Vec<(String, (u64, u64))>, std::io::Error> {
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    let mut stats = Vec::new();
    let re = Regex::new(r"^\s*([^:]+):\s*(\d+)\s+.*\s+(\d+)\s+").unwrap();

    for line in reader.lines().skip(2) {
        // Skip the header lines
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            let interface = caps[1].to_string();
            let rx_bytes: u64 = caps[2].parse().unwrap_or(0);
            let tx_bytes: u64 = caps[3].parse().unwrap_or(0);
            stats.push((interface, (rx_bytes, tx_bytes)));
        }
    }
    Ok(stats)
}
