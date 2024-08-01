use std::collections::HashMap;

#[macro_export]
macro_rules! test_debug {
    ($($arg:tt)*) => {{
        if std::env::var("RUST_TEST").is_ok() {
            println!($($arg)*);
        }
    }};
}

/// Filters out interfaces that have zero RX and TX counters.
pub fn filter_zero_counters(
    stats: &HashMap<String, (u64, u64)>,
    interfaces: &[String],
) -> Vec<String> {
    interfaces
        .iter()
        .filter(|iface| {
            if let Some(&(rx, tx)) = stats.get(*iface) {
                rx != 0 || tx != 0
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

/// Shortens the interface name if it exceeds 16 characters, following specific rules.
pub fn shorten_name(name: &str) -> String {
    if name.len() > 16 {
        let start = name.find('\\').map(|i| i + 1).unwrap_or(0);
        let name = &name[start..];

        // assume form like \DEVICE\TCPIP_{2EE2C70C-A092-4D88-A654-98C8D7645CD5}
        if let Some(start_idx) = name.find("TCPIP_{") {
            let prefix_len = start_idx + 11; // length of "TCPIP_{0737"
            if prefix_len < name.len() {
                let suffix_start = name.len().saturating_sub(5);
                let prefix = &name[start_idx..prefix_len];
                let suffix = &name[suffix_start..];

                return format!("{}..{}", prefix, suffix);
            }
        }

        // If the name doesn't match the expected pattern or prefix_len check fails
        if name.len() > 13 {
            return format!("{}...", &name[..13]);
        }
    }
    // If the name length is 16 or less, or all other conditions fail
    name.to_string()
}

/// Prints headers for the network interface statistics table.
pub fn print_headers(
    interfaces: &[String],
    writer: &mut dyn std::io::Write,
    hide_zero_counters: bool,
    stats: &HashMap<String, (u64, u64)>,
) -> std::io::Result<()> {
    let interfaces = if hide_zero_counters {
        filter_zero_counters(stats, interfaces)
    } else {
        interfaces.to_vec()
    };

    if interfaces.is_empty() {
        return Ok(());
    }

    let width = 18; // Width for each interface field including space for in/out
    for (i, interface) in interfaces.iter().enumerate() {
        let short_interface = shorten_name(interface);
        let padded_name = format!("{:^width$}", short_interface, width = width);
        write!(writer, "{}", padded_name)?;
        if i < interfaces.len() - 1 {
            write!(writer, "  ")?; // Additional spaces between columns
        }
    }
    writeln!(writer)?;

    for (i, _) in interfaces.iter().enumerate() {
        write!(writer, "{:>8}  {:>8}", "KB/s in", "KB/s out")?;
        if i < interfaces.len() - 1 {
            write!(writer, "  ")?; // Additional spaces between columns
        }
    }
    writeln!(writer)?;

    Ok(())
}

/// Prints the network interface statistics.
pub fn print_stats(
    previous: &HashMap<String, (u64, u64)>,
    current: &HashMap<String, (u64, u64)>,
    interfaces: &[String],
    writer: &mut dyn std::io::Write,
    hide_zero_counters: bool,
) -> std::io::Result<()> {
    let interfaces = if hide_zero_counters {
        filter_zero_counters(current, interfaces)
    } else {
        interfaces.to_vec()
    };

    for (i, interface) in interfaces.iter().enumerate() {
        if let (Some(&(prev_rx, prev_tx)), Some(&(cur_rx, cur_tx))) =
            (previous.get(interface), current.get(interface))
        {
            let rx_kbps = (cur_rx.saturating_sub(prev_rx)) as f64 / 1024.0;
            let tx_kbps = (cur_tx.saturating_sub(prev_tx)) as f64 / 1024.0;
            write!(writer, "{:>8.2}  {:>8.2}", rx_kbps, tx_kbps)?;
            if i < interfaces.len() - 1 {
                write!(writer, "  ")?; // Additional spaces between columns
            }
        }
    }
    writeln!(writer)?;

    Ok(())
}
