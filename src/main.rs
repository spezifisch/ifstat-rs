mod net_stats;
mod opts;
mod output;

use clap::Parser;
use net_stats::get_net_dev_stats;
use opts::Opts;
use output::{print_headers, print_net_devices, print_stats};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Parse command-line options
    let opts: Opts = Opts::parse();

    if opts.list_interfaces {
        // List interface names and exit.
        match get_net_dev_stats() {
            Ok(stats) => print_net_devices(&stats),
            Err(e) => eprintln!("Error listing network interfaces: {}", e),
        }
        return;
    }

    let interfaces: Vec<String> = opts
        .interfaces
        .clone()
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    // Get initial network statistics
    let mut previous_stats = get_net_dev_stats().expect("Failed reading network interface stats.");

    // Determine which interfaces to monitor
    let monitor_interfaces: Vec<String> = if opts.monitor_all {
        previous_stats.keys().cloned().collect()
    } else if opts.interfaces.is_some() {
        interfaces
    } else if opts.monitor_loopback {
        previous_stats.keys().cloned().collect()
    } else {
        previous_stats
            .keys()
            .filter(|iface| !iface.starts_with("lo"))
            .cloned()
            .collect()
    };

    // Print headers based on specified or available interfaces
    let header_repeat_interval = 20;
    print_headers(
        &monitor_interfaces,
        &mut std::io::stdout(),
        opts.hide_zero_counters,
        &previous_stats,
    )
    .unwrap();

    // Use first_measurement delay if provided, otherwise use delay
    let first_delay = opts.first_measurement.unwrap_or(opts.delay);
    let regular_delay = opts.delay;

    // Sleep for the first delay
    sleep(Duration::from_secs_f64(first_delay)).await;

    let mut updates = 0;
    let mut lines_since_last_header = 0;

    loop {
        // Check if the number of updates has reached the specified count
        if let Some(count) = opts.count {
            if updates >= count {
                break;
            }
        }

        // Get current network statistics
        match get_net_dev_stats() {
            Ok(current_stats) => {
                // Print headers again if enough lines have been printed
                if lines_since_last_header >= header_repeat_interval {
                    print_headers(
                        &monitor_interfaces,
                        &mut std::io::stdout(),
                        opts.hide_zero_counters,
                        &current_stats,
                    )
                    .unwrap();
                    lines_since_last_header = 0;
                }

                // Print stats for the monitored interfaces
                print_stats(
                    &previous_stats,
                    &current_stats,
                    &monitor_interfaces,
                    &mut std::io::stdout(),
                    opts.hide_zero_counters,
                )
                .unwrap();
                previous_stats = current_stats;

                lines_since_last_header += 1;
            }
            Err(e) => eprintln!("Error reading network statistics: {}", e),
        }

        updates += 1;

        // Sleep for the regular delay
        sleep(Duration::from_secs_f64(regular_delay)).await;
    }
}
