use clap::Parser;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tokio::time::{sleep, Duration};

#[derive(Parser)]
#[clap(version = "1.0", author = "Your Name")]
struct Opts {
    /// Interfaces to monitor, separated by commas (e.g., "eth0,lo")
    #[clap(short, long)]
    interfaces: Option<String>,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let interfaces: Vec<String> = opts
        .interfaces
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let mut previous_stats = get_net_dev_stats().unwrap();

    print_headers(&interfaces);

    loop {
        sleep(Duration::from_secs(1)).await;
        match get_net_dev_stats() {
            Ok(current_stats) => {
                print_stats(&previous_stats, &current_stats, &interfaces);
                previous_stats = current_stats;
            }
            Err(e) => eprintln!("Error reading /proc/net/dev: {}", e),
        }
    }
}

fn get_net_dev_stats() -> Result<HashMap<String, (u64, u64)>, std::io::Error> {
    let file = File::open("/proc/net/dev")?;
    let reader = BufReader::new(file);
    let mut stats = HashMap::new();
    let re = Regex::new(r"^\s*([^:]+):\s*(\d+)\s+.*\s+(\d+)\s+").unwrap();

    for line in reader.lines().skip(2) {
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            let interface = caps[1].to_string();
            let rx_bytes: u64 = caps[2].parse().unwrap_or(0);
            let tx_bytes: u64 = caps[3].parse().unwrap_or(0);
            stats.insert(interface, (rx_bytes, tx_bytes));
        }
    }
    Ok(stats)
}

fn print_headers(interfaces: &[String]) {
    if interfaces.is_empty() {
        return;
    }

    print!("        ");
    for interface in interfaces {
        print!("{:<20}", interface);
    }
    println!();

    print!("        ");
    for _ in interfaces {
        print!("{:<10} {:<10}", "KB/s in", "KB/s out");
    }
    println!();
}

fn print_stats(
    previous: &HashMap<String, (u64, u64)>,
    current: &HashMap<String, (u64, u64)>,
    interfaces: &[String],
) {
    let interface_names: Vec<_> = if interfaces.is_empty() {
        current.keys().cloned().collect()
    } else {
        interfaces.to_vec()
    };

    if interface_names.is_empty() {
        return;
    }

    for interface in &interface_names {
        if let (Some(&(prev_rx, prev_tx)), Some(&(cur_rx, cur_tx))) =
            (previous.get(interface), current.get(interface))
        {
            let rx_kbps = (cur_rx.saturating_sub(prev_rx)) as f64 / 1024.0;
            let tx_kbps = (cur_tx.saturating_sub(prev_tx)) as f64 / 1024.0;
            print!("{:<10.2} {:<10.2} ", rx_kbps, tx_kbps);
        }
    }
    println!();
}
