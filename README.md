# ifstat-rs

"This project has been generated with the assistance of ChatGPT. Please be aware that ChatGPT can make mistakes. It is important to review and verify all information provided by the tool." is what it says. Don't use this tool.

`ifstat-rs` is a simple Rust tool to parse and display network I/O statistics from `/proc/net/dev` once a second.

## Features

- Parses network interface statistics from `/proc/net/dev`
- Displays RX and TX bytes for each interface every second

## Installation

To install `ifstat-rs`, ensure you have Rust and Cargo installed, then run:

```sh
cargo build --release --target x86_64-unknown-linux-gnu
```

## Usage

Run the tool with:

```sh
cargo run --release --target x86_64-unknown-linux-gnu
```

You should see output similar to:

```text
Interface: eth0, RX: 123456 bytes, TX: 789012 bytes
Interface: wlan0, RX: 345678 bytes, TX: 901234 bytes
```

## Contributing

1. Fork the repository
2. Create a new branch (`git checkout -b feature-branch`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature-branch`)
5. Create a new Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
