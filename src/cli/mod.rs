use crate::api::search::{parse_duration, SearchRequest, TimeRange};
use crate::output::OutputFormat;
use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};

/// grayterm — CLI for querying Graylog logs.
///
/// <purpose-start>
/// Top-level clap Parser struct. Defines the command tree and delegates
/// argument parsing to sub-structs. Keeping the root struct thin makes it
/// easy to add new subcommands without changing dispatch logic.
/// [initial-implementation.md]
/// <purpose-end>
#[derive(Debug, Parser)]
#[command(name = "grayterm", version, about = "Query Graylog logs via REST API")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level subcommands for grayterm.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Search Graylog messages.
    Search(Box<SearchArgs>),
    /// Manage Graylog streams.
    #[command(subcommand)]
    Streams(StreamsCommand),
}

/// Subcommands under `grayterm streams`.
#[derive(Debug, Subcommand)]
pub enum StreamsCommand {
    /// List available Graylog streams.
    List(StreamsListArgs),
}

/// Arguments for `grayterm search`.
///
/// <purpose-start>
/// Captures all search parameters from the command line and exposes
/// `into_search_request()` to convert them into the API-facing SearchRequest.
/// Validation (mutual exclusion, defaults) lives here rather than in the
/// HTTP client.
/// [initial-implementation.md]
/// <purpose-end>
#[derive(Debug, Args)]
pub struct SearchArgs {
    /// Graylog search query (Lucene syntax). Defaults to all messages.
    #[arg(short = 'q', long, default_value = "*")]
    pub query: String,

    /// Stream ID to search (repeatable). Defaults to all streams.
    #[arg(short = 's', long = "stream", value_name = "ID")]
    pub streams: Vec<String>,

    /// Relative time window, e.g. 1h, 30m, 2d. Mutually exclusive with --from/--to.
    #[arg(long, conflicts_with_all = ["from", "to"], value_name = "DURATION")]
    pub last: Option<String>,

    /// Absolute start time (RFC3339). Requires --to.
    #[arg(long, requires = "to", conflicts_with = "last", value_name = "RFC3339")]
    pub from: Option<String>,

    /// Absolute end time (RFC3339). Requires --from.
    #[arg(long, requires = "from", conflicts_with = "last", value_name = "RFC3339")]
    pub to: Option<String>,

    /// Field to include in results (repeatable). Defaults to timestamp, source, message.
    #[arg(short = 'f', long = "field", value_name = "NAME")]
    pub fields: Vec<String>,

    /// Maximum number of messages to return.
    #[arg(short = 'n', long, value_name = "N")]
    pub limit: Option<u64>,

    /// Field to sort results by.
    #[arg(long, value_name = "FIELD")]
    pub sort: Option<String>,

    /// Sort direction.
    #[arg(long, value_name = "asc|desc", default_value = "desc")]
    pub sort_order: String,

    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

/// Arguments for `grayterm streams list`.
#[derive(Debug, Args)]
pub struct StreamsListArgs {
    /// Output format.
    #[arg(long, value_enum, default_value = "text")]
    pub format: OutputFormat,
}

impl SearchArgs {
    /// Converts CLI arguments into a SearchRequest, applying validation and defaults.
    ///
    /// <purpose-start>
    /// Validates the time-range mutual exclusion (--last vs --from/--to),
    /// defaults to the last hour when no range is specified, and falls back to
    /// the standard fields when none are explicitly requested.
    /// [initial-implementation.md]
    /// <purpose-end>
    ///
    /// <outputs-start>
    /// Returns a SearchRequest ready for the HTTP client, or an error if
    /// arguments are invalid (e.g. an unparseable duration).
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// None.
    /// <side-effects-end>
    pub fn into_search_request(self) -> Result<SearchRequest> {
        let timerange = match (self.last, self.from, self.to) {
            (Some(d), None, None) => TimeRange::Relative { range: parse_duration(&d)? },
            (None, Some(from), Some(to)) => TimeRange::Absolute { from, to },
            (None, None, None) => TimeRange::Relative { range: 3_600 }, // default: last 1 h
            _ => bail!("Specify either --last or --from/--to, not both"),
        };

        let fields = if self.fields.is_empty() {
            vec!["timestamp".into(), "source".into(), "message".into()]
        } else {
            self.fields
        };

        // Only include sort_order when a sort field is present; omit both when
        // the sort field is absent so the server applies its own default ordering.
        let sort_order = self.sort.as_ref().map(|_| self.sort_order);

        Ok(SearchRequest {
            query: self.query,
            streams: self.streams,
            fields,
            timerange,
            size: self.limit,
            sort: self.sort,
            sort_order,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_search(args: &[&str]) -> Result<SearchArgs, clap::Error> {
        let full: Vec<&str> =
            std::iter::once("grayterm").chain(std::iter::once("search")).chain(args.iter().copied()).collect();
        Cli::try_parse_from(full).map(|cli| match cli.command {
            Command::Search(a) => *a,
            _ => panic!("expected Search"),
        })
    }

    #[test]
    fn no_time_args_defaults_to_last_1h() {
        let req = parse_search(&[]).unwrap().into_search_request().unwrap();
        assert_eq!(req.timerange, TimeRange::Relative { range: 3_600 });
    }

    #[test]
    fn last_flag_produces_relative_timerange() {
        let req = parse_search(&["--last", "30m"]).unwrap().into_search_request().unwrap();
        assert_eq!(req.timerange, TimeRange::Relative { range: 1_800 });
    }

    #[test]
    fn from_to_produce_absolute_timerange() {
        let req = parse_search(&[
            "--from", "2026-06-12T00:00:00Z",
            "--to",   "2026-06-12T01:00:00Z",
        ])
        .unwrap()
        .into_search_request()
        .unwrap();
        assert_eq!(
            req.timerange,
            TimeRange::Absolute {
                from: "2026-06-12T00:00:00Z".into(),
                to: "2026-06-12T01:00:00Z".into(),
            }
        );
    }

    #[test]
    fn no_fields_given_uses_defaults() {
        let req = parse_search(&[]).unwrap().into_search_request().unwrap();
        assert_eq!(req.fields, ["timestamp", "source", "message"]);
    }

    #[test]
    fn explicit_fields_replace_defaults() {
        let req = parse_search(&["--field", "level", "--field", "message"])
            .unwrap()
            .into_search_request()
            .unwrap();
        assert_eq!(req.fields, ["level", "message"]);
    }

    #[test]
    fn last_and_from_are_mutually_exclusive() {
        assert!(parse_search(&["--last", "1h", "--from", "2026-06-12T00:00:00Z"]).is_err());
    }

    #[test]
    fn from_without_to_is_rejected() {
        assert!(parse_search(&["--from", "2026-06-12T00:00:00Z"]).is_err());
    }

    #[test]
    fn sort_order_omitted_when_no_sort_field() {
        let req = parse_search(&[]).unwrap().into_search_request().unwrap();
        assert!(req.sort_order.is_none());
    }

    #[test]
    fn sort_order_included_with_sort_field() {
        let req = parse_search(&["--sort", "timestamp", "--sort-order", "asc"])
            .unwrap()
            .into_search_request()
            .unwrap();
        assert_eq!(req.sort.as_deref(), Some("timestamp"));
        assert_eq!(req.sort_order.as_deref(), Some("asc"));
    }
}
