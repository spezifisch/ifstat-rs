# ifstat-rs

`ifstat-rs` is a straightforward and efficient Rust tool to report network interface stats just like [ifstat](http://gael.roualland.free.fr/ifstat/).

[![tests](https://github.com/spezifisch/ifstat-rs/actions/workflows/test.yml/badge.svg)](https://github.com/spezifisch/ifstat-rs/actions/workflows/test.yml)
[![builds](https://github.com/spezifisch/ifstat-rs/actions/workflows/build.yml/badge.svg)](https://github.com/spezifisch/ifstat-rs/actions/workflows/build.yml)

## Features

- **Real-time Monitoring:** Parses and displays RX and TX bytes for each network interface every second.
- **Interface Flexibility:** Monitor specific interfaces, all interfaces, or just loopback interfaces.
- **Configurable Updates:** Set delays between updates and limit the number of updates.

## Installation

To install `ifstat-rs`, ensure you have Rust and Cargo installed, then run:

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
- `--first-measurement <seconds>`: Set delay before the first measurement (default is the same as --delay).
- `<delay>`: Delay between updates in seconds (default is 1 second).
- `<count>`: Number of updates before stopping (default is unlimited).

### Examples

Monitor specific interfaces:

```sh
ifstat-rs -i eth0,wlan1
```

Monitor all interfaces:

```sh
ifstat-rs -a 1 10
```

Include loopback interfaces:

```sh
ifstat-rs -l
```

Specify delay and count:

```sh
ifstat-rs -a --first-measurement 1 60 10
```

### Development

Run the tool with:

```sh
cargo run -- -a
```

## Supported Platforms

- **Linux:** Fully supported with real-time network statistics from `/proc/net/dev`.
- **macOS:** Provisions in place, support in development.
- **Windows:** Provisions in place, support in development.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

*"This project has been generated with the assistance of ChatGPT. Please be aware that ChatGPT can make mistakes. It is important to review and verify all information provided by the tool."* is what it says. Just so you know. Because I guess maybe all the bugs aren't my fault after all.
