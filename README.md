# ifstat-rs

`ifstat-rs` is a straightforward and efficient Rust tool to report network interface stats, similar to [ifstat](http://gael.roualland.free.fr/ifstat/).

[![tests](https://github.com/spezifisch/ifstat-rs/actions/workflows/test.yml/badge.svg)](https://github.com/spezifisch/ifstat-rs/actions/workflows/test.yml)
[![builds](https://github.com/spezifisch/ifstat-rs/actions/workflows/build.yml/badge.svg)](https://github.com/spezifisch/ifstat-rs/actions/workflows/build.yml)

## Features

- **Real-time Monitoring:** Parses and displays RX and TX bytes for each network interface every second.
- **Interface Flexibility:** Monitor specific interfaces, all interfaces, or just loopback interfaces.
- **Configurable Updates:** Set delays between updates and limit the number of updates.
- **Interface Listing with Friendly Names:** Displays friendly names for interfaces where supported (currently Windows).

## Supported Platforms

- **Linux:** Fully supported with real-time network statistics from `/proc/net/dev`.
- **Windows:** Supported as of v2.0.0.
- **macOS:** Supported as of v3.0.0.

## Installation

To install `ifstat-rs`, ensure you have Rust and Cargo installed.

### Release from crates.io

```sh
cargo install ifstat-rs
```

### Install from Development Git Repository

```sh
cargo build --release
sudo cp target/release/ifstat-rs /usr/local/bin/
ifstat-rs --version
```

## Usage

### Options

**ifstat-rs**

- `-a`: Monitor all interfaces.
- `-l`: Include loopback interfaces.
- `-i <interfaces>`: Specify interfaces to monitor, separated by commas (e.g., `-i eth0,lo`).
- `--list-interfaces`: List all interfaces, with their friendly names where supported (currently Windows).
- `--first-measurement <seconds>`: Set delay before the first measurement (default is the same as --delay).
- `<delay>`: Delay between updates in seconds (default is 1 second).
- `<count>`: Number of updates before stopping (default is unlimited).

### Examples

Monitor specific interfaces:

```sh
# Linux/Mac-style
ifstat-rs -i eth0,wlan1
# Windows-style
ifstat-rs -i "\DEVICE\TCPIP_{66963456-C690-4E4E-940B-E7C915B9A07D},\DEVICE\TCPIP_{97D92124-3AC3-45B5-8634-F6547F9676CE}"
```

Lookup list of interfaces, (on Windows: with Adapter Names where present):

```console
ifstat-rs --list-interfaces
 3 adapters:
\DEVICE\TCPIP_{66963456-C690-4E4E-940B-E7C915B9A07D} Ethernet
\DEVICE\TCPIP_{3CE6ABDA-3928-11EF-BFD9-806E6F6E6963} Loopback Pseudo-Interface 1
\DEVICE\TCPIP_{97D92124-3AC3-45B5-8634-F6547F9676CE} vEthernet (nat)
16 interfaces:
[...]
\DEVICE\TCPIP_{97D92124-3AC3-45B5-8634-F6547F9676CE} (vEthernet (nat))
\DEVICE\TCPIP_{66963456-C690-4E4E-940B-E7C915B9A07D} (Ethernet)
\DEVICE\TCPIP_{34DB3D4C-C1FC-4CDF-8BB2-9A7B1D40D640}
\DEVICE\TCPIP_{3CE6ABDA-3928-11EF-BFD9-806E6F6E6963} (Loopback Pseudo-Interface 1)
[...]
```

Monitor all interfaces, measure every second, for 10 seconds:

```sh
ifstat-rs -a 1 10
```

Include loopback interfaces:

```sh
ifstat-rs -l
```

Specify delay and count - first measure after 1 second, then every 60 seconds:

```sh
ifstat-rs -a --first-measurement 1 60
```

### Development

Run the tool with:

```sh
cargo run -- -a
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

*"This project has been generated with the assistance of ChatGPT. Please be aware that ChatGPT can make mistakes. It is important to review and verify all information provided by the tool."* is what it says. Just so you know. Because I guess maybe all the bugs aren't my fault after all.
