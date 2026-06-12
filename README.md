# grayterm

Self-contained CLI for viewing Graylog logs via REST API, written in Rust. Suitable for agentic usage.

## Features

- Search Graylog messages via the Search Scripting API (`POST /api/search/messages`)
- Filter logs by time range (relative or absolute), stream, and Lucene query
- Output formats: plain text, CSV, JSON — rendered server-side and streamed to stdout
- List Graylog streams with their IDs
- Token-based authentication
- Configurable via environment variables and config file
- No runtime dependencies — single static binary

## Installation

```sh
cargo install --path .
```

## Usage

```sh
# Search last hour of error logs (text output)
grayterm search --query "level:ERROR" --last 1h

# Search in a specific stream and get JSON output
grayterm search --stream <stream-id> --last 30m --format json | jq .

# Absolute time range with custom fields and CSV output
grayterm search \
  --from 2026-06-12T00:00:00Z \
  --to   2026-06-12T01:00:00Z \
  --field timestamp --field source --field message \
  --format csv

# List streams
grayterm streams list

# List streams as JSON
grayterm streams list --format json
```

### `search` options

| Flag | Description | Default |
|------|-------------|---------|
| `-q, --query` | Lucene search query | `*` |
| `-s, --stream` | Stream ID (repeatable) | all streams |
| `--last` | Relative window, e.g. `1h`, `30m`, `2d` | `1h` if no range given |
| `--from` | Absolute start (RFC3339) — requires `--to` | — |
| `--to` | Absolute end (RFC3339) — requires `--from` | — |
| `-f, --field` | Field to include (repeatable) | `timestamp source message` |
| `-n, --limit` | Max messages returned | — |
| `--sort` | Field to sort by | — |
| `--sort-order` | `asc` or `desc` | `desc` (when `--sort` is set) |
| `--format` | `text`, `csv`, or `json` | `text` |

## Configuration

Set credentials via environment variables or a config file at `~/.config/grayterm/config.toml`:

```toml
[graylog]
url   = "https://graylog.example.com"
token = "your-personal-access-token"
```

Environment variables override config file values:

| Variable         | Description                       |
|------------------|-----------------------------------|
| `GRAYTERM_URL`   | Graylog server URL                |
| `GRAYTERM_TOKEN` | Graylog personal access token     |

### Creating a Graylog access token

In the Graylog web UI: **System → Users → Edit user → Access Tokens → Create token**.
The token is used as the HTTP Basic auth username; the literal string `token` is the password
(Graylog's documented convention).

## Development

```sh
cargo build
cargo test
cargo clippy
```
