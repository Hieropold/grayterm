# grayterm

Self-contained CLI for viewing Graylog logs via REST API, written in Rust. Suitable for agentic usage.

## Features

- Query Graylog streams and messages via REST API
- Filter logs by time range, stream, and search query
- Output formats: plain text, JSON (for piping into other tools)
- Configurable via environment variables and config file
- No runtime dependencies — single static binary

## Installation

```sh
cargo install --path .
```

## Usage

```sh
# Search logs
grayterm search --query "level:ERROR" --last 1h

# List streams
grayterm streams list

# Tail logs (poll for new messages)
grayterm tail --stream <stream-id>
```

## Configuration

Set credentials via environment variables or a config file at `~/.config/grayterm/config.toml`:

```toml
[graylog]
url = "https://graylog.example.com"
username = "admin"
password = "secret"
```

Environment variables override config file values:

| Variable              | Description              |
|-----------------------|--------------------------|
| `GRAYTERM_URL`        | Graylog server URL       |
| `GRAYTERM_USERNAME`   | API username             |
| `GRAYTERM_PASSWORD`   | API password or token    |

## Development

```sh
cargo build
cargo test
cargo clippy
```
