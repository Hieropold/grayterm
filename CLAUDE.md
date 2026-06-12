# grayterm — AI Agent Instructions

## Project Overview

`grayterm` is a self-contained Rust CLI for querying Graylog logs via the Graylog REST API. It is designed to be scriptable and suitable for agentic usage — structured output, non-interactive by default, and composable with standard Unix tools.

See [docs/GLOSSARY.md](docs/GLOSSARY.md) for the project's domain vocabulary.

---

## Constitution Principles (Non-Negotiable)

1. **Performance-Focused** — minimize CPU and memory usage without sacrificing correctness. Prefer zero-copy parsing and streaming over buffering full responses.
2. **Test-Driven Development** — all new features require comprehensive automated tests.
3. **Check Docs Before Using Libraries** — always verify up-to-date documentation before introducing or updating a dependency. Use Context7 MCP to check docs and examples.
4. **Semantic Markup Doc Comments** — mandatory on every new or modified function/struct/logical block (see below).

---

## Change Summary Docs

Track all changes in the `docs/` folder. Each change must have a summary document named by GitHub issue id, e.g. `docs/42.md`, containing what changed and why. Check this as the last step of any implementation.

Change summary docs must be referenced in relevant semantic markup blocks.

---

## Semantic Markup Doc Comments (Mandatory)

Every new or modified function, method, struct, or logical block **must** have a doc comment with the following sections. Focus on **WHY**, not HOW.

```rust
/// Short one-line summary of what this does.
///
/// <purpose-start>
/// Why this exists and what problem it solves.
/// [42.md] Reference to change summary doc if applicable.
/// <purpose-end>
///
/// <inputs-start>
/// - `param_name`: Description of the parameter.
/// <inputs-end>
///
/// <outputs-start>
/// Returns description of return value and meaning.
/// <outputs-end>
///
/// <side-effects-start>
/// - Any I/O, mutations, or observable state changes.
/// <side-effects-end>
```

Rules:
- Comments must be in **English**.
- Use Rust `///` doc comment style.
- When editing existing code, read and preserve existing doc comments; update them only if behavior changes.
- Do **not** remove existing doc comments unless they are being replaced.

---

## Stack

- **Language**: Rust (stable toolchain)
- **HTTP client**: `reqwest` (async, with rustls)
- **Async runtime**: `tokio`
- **CLI parsing**: `clap` (derive API)
- **Serialization**: `serde` + `serde_json`
- **Config**: `config` crate or manual TOML parsing with `toml`
- **Error handling**: `anyhow` for application errors, `thiserror` for library errors

---

## Architecture & Folder Conventions

```
src/
  main.rs          # Entry point, CLI dispatch
  cli/             # clap command definitions
  api/             # Graylog REST API client
  config.rs        # Configuration loading (env + file)
  output/          # Output formatters (text, JSON)
  error.rs         # Error types
docs/              # Change summary documents (one per GitHub issue)
tests/             # Integration tests
```

---

## Development Rules

- Language: **Rust only** — no other languages in the implementation.
- All changes must go through pull requests with at least one approval.
- Do not skip pre-commit hooks or bypass signing.
- Do not add features beyond what is explicitly requested.
- Do not add error handling for scenarios that cannot happen.
- Use Context7 MCP to check docs and examples before using a crate.
- **Git write operations are handled by the user** — do not run `git add`, `git commit`, or `git push`. Only use git for reads (`git status`, `git log`, `git diff`). The user stages and commits manually.
