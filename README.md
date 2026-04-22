# qrcon

A lightweight, high-performance Quake RCON (Remote Console) client written in Rust. Designed as a fast, reliable, and script-friendly replacement for `icecon`.

## Features

- **Fast:** Extremely low overhead for one-off commands.
- **Robust:** Built-in timeout and retry logic.
- **Interactive:** Includes a shell mode for continuous command execution.
- **Cross-platform:** Support for Windows and Linux.

## Installation

Download the latest binary from the [Releases](https://github.com/sindicatorp/qrcon/releases) page.

## Usage

### Single Command
```bash
qrcon <address:port> <password> -c <command>
```

### Interactive Shell
```bash
qrcon <address:port> <password>
```

### Options
- `-t, --timeout <seconds>`: Timeout for each request (default: 2)
- `-r, --retries <count>`: Number of retries on timeout (default: 3)

## Benchmarks

`qrcon` is significantly faster than `icecon` for one-off commands:
- **qrcon:** ~0.005s per command
- **icecon:** ~0.084s per command

## License
MIT
