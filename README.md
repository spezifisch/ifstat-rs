# ifstat-rs

`ifstat-rs` is a simple Rust tool to parse and display network I/O statistics from `/proc/net/dev` once a second. 

*"This project has been generated with the assistance of ChatGPT. Please be aware that ChatGPT can make mistakes. It is important to review and verify all information provided by the tool." is what it says. Don't use this tool.*

## Features

- Parses network interface statistics from `/proc/net/dev`
- Displays RX and TX bytes for each interface every second
- Supports monitoring specific interfaces, all interfaces, or loopback interfaces
- Allows setting delay between updates and limiting the number of updates

## Installation

To install `ifstat-rs`, ensure you have Rust and Cargo installed, then run:

```sh
cargo build --release
```

## Usage

Run the tool with:

```sh
cargo run --release
```

### Options

- `-a`: Monitor all interfaces
- `-l`: Include loopback interfaces
- `-i <interfaces>`: Specify interfaces to monitor, separated by commas (e.g., `-i eth0,lo`)
- `--delay <seconds>`: Set delay between updates (default is 1 second)
- `<count>`: Number of updates before stopping (default is unlimited)

### Examples

Monitor specific interfaces:

```sh
cargo run --release -- -i eth0,lo
```

Monitor all interfaces:

```sh
cargo run --release -- -a
```

Include loopback interfaces:

```sh
cargo run --release -- -l
```

Specify delay and count:

```sh
cargo run --release -- -a --delay 0.5 10
```

## Contributing

1. Fork the repository
2. Create a new branch (`git checkout -b feature-branch`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature-branch`)
5. Create a new Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.