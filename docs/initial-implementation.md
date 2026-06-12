# Initial Implementation — grayterm v0.1.0

## What changed

Implemented the first working version of grayterm: a scriptable Rust CLI for
querying Graylog logs and listing streams via the Graylog REST API.

### Commands added

- **`grayterm search`** — searches messages via `POST /api/search/messages`
  (Graylog Search Scripting API). Accepts a Lucene query, one or more stream
  IDs, a relative (`--last`) or absolute (`--from`/`--to`) time range, field
  selection, result limit, sort, and output format. Response is streamed
  byte-for-byte to stdout (zero-copy passthrough); the Graylog server renders
  text/CSV/JSON before the data leaves the server.

- **`grayterm streams list`** — fetches `GET /api/streams` and either prints
  tab-separated `<id>\t<title>` lines (text mode) or forwards the raw JSON
  response to stdout (json mode).

### Authentication

Token-based HTTP Basic auth: the access token is the Basic auth username;
the literal string `"token"` is the password. This is the Graylog personal
access token convention. `X-Requested-By: grayterm` is added to all non-GET
requests to satisfy Graylog's CSRF protection requirement.

### Configuration

Config file: `~/.config/grayterm/config.toml` (XDG-aware).  
Environment overrides: `GRAYTERM_URL`, `GRAYTERM_TOKEN`.  
Missing required values produce a clear error message naming the missing field.

### Module structure

```
src/
  lib.rs           — re-exports all modules; enables integration tests
  main.rs          — tokio::main entry point, CLI dispatch
  cli/mod.rs       — clap derive CLI (Cli, Command, SearchArgs, StreamsListArgs)
  config.rs        — Config loading with file + env merge
  api/
    mod.rs         — re-exports GraylogClient
    client.rs      — GraylogClient: reqwest wrapper with token auth
    search.rs      — SearchRequest, TimeRange, duration parser, body builder
    streams.rs     — StreamsResponse, format_streams_text
  output/mod.rs    — OutputFormat enum → Accept header mapping
  error.rs         — ApiError (thiserror)
tests/
  search_integration.rs   — wiremock integration tests for search
  streams_integration.rs  — wiremock integration tests for streams list
```

## Why

The initial scaffolding was a bare `Hello, world!` with no dependencies.
This change delivers the first real feature set: everything needed to query
Graylog from a terminal or script.

Design decisions:
- **Search Scripting API** (`POST /api/search/messages`) chosen over the
  deprecated Universal Search because it is the official modern endpoint and
  is actively maintained in Graylog 5/6.
- **Stream passthrough output** for search results: zero CPU/memory overhead
  per the Performance-Focused constitution principle.
- `lib.rs` added alongside `main.rs` so integration tests can import types
  as an external consumer.
- `Box<SearchArgs>` in the `Command` enum to avoid large-variant clippy lint.
