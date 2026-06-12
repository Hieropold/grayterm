use anyhow::Result;
use clap::Parser;
use grayterm::api::GraylogClient;
use grayterm::cli::{Cli, Command, StreamsCommand};
use grayterm::config::Config;

/// Entry point for the grayterm CLI.
///
/// <purpose-start>
/// Parses command-line arguments, loads configuration, constructs the Graylog
/// client, and dispatches to the appropriate command handler. Errors surface as
/// a non-zero exit code and a human-readable message via anyhow's Result impl
/// for main.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <side-effects-start>
/// - Reads configuration from a file and environment variables.
/// - Makes HTTP requests to the configured Graylog instance.
/// - Writes output to stdout.
/// - Exits with a non-zero code on error.
/// <side-effects-end>
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;
    let client = GraylogClient::new(config.url, config.token)?;

    match cli.command {
        Command::Search(args) => {
            let format = args.format;
            let req = args.into_search_request()?;
            client.search(&req, format).await?;
        }
        Command::Streams(StreamsCommand::List(args)) => {
            client.streams_list(args.format).await?;
        }
    }

    Ok(())
}
