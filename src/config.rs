use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// Resolved runtime configuration for a grayterm session.
///
/// <purpose-start>
/// Holds the Graylog server URL and access token after merging the config
/// file and environment variables. Using a resolved struct rather than
/// reading env/file at call sites prevents scattered configuration logic
/// and ensures a single validation point.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <side-effects-start>
/// None once constructed.
/// <side-effects-end>
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL of the Graylog instance, e.g. `https://graylog.example.com`.
    pub url: String,
    /// Personal access token for HTTP Basic auth.
    pub token: String,
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    graylog: Option<GraylogSection>,
}

#[derive(Debug, Deserialize, Default)]
struct GraylogSection {
    url: Option<String>,
    token: Option<String>,
}

/// Returns the path to the grayterm config file, respecting XDG conventions.
///
/// <purpose-start>
/// Centralises config-file location logic so it follows the XDG Base
/// Directory spec: $XDG_CONFIG_HOME takes priority, falling back to
/// $HOME/.config.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <outputs-start>
/// Returns Some(path) if the home/XDG directory can be determined, None
/// otherwise.
/// <outputs-end>
///
/// <side-effects-start>
/// Reads XDG_CONFIG_HOME and HOME from the process environment.
/// <side-effects-end>
fn config_file_path() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|base| base.join("grayterm").join("config.toml"))
}

/// Reads the config file and returns the raw (url, token) pair from it.
///
/// <purpose-start>
/// Isolates file I/O so Config::load() can be tested without a real file
/// by using Config::from_parts() directly.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <outputs-start>
/// Returns Ok((None, None)) when the file does not exist, or the values
/// found inside it.
/// <outputs-end>
///
/// <side-effects-start>
/// - May read a file from disk.
/// <side-effects-end>
fn read_config_file() -> Result<(Option<String>, Option<String>)> {
    let Some(path) = config_file_path() else {
        return Ok((None, None));
    };
    if !path.exists() {
        return Ok((None, None));
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    let cfg: FileConfig = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
    let g = cfg.graylog.unwrap_or_default();
    Ok((g.url, g.token))
}

impl Config {
    /// Loads configuration from the config file and environment variables.
    ///
    /// <purpose-start>
    /// Merges the optional config file with environment variable overrides,
    /// applying the precedence: env > file. Fails with a clear error if either
    /// required field is absent after the merge.
    /// [initial-implementation.md]
    /// <purpose-end>
    ///
    /// <outputs-start>
    /// Returns a fully resolved Config or an error naming the missing field.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - May read ~/.config/grayterm/config.toml.
    /// - Reads GRAYTERM_URL and GRAYTERM_TOKEN from the environment.
    /// <side-effects-end>
    pub fn load() -> Result<Self> {
        let (mut url, mut token) = read_config_file()?;
        if let Ok(v) = std::env::var("GRAYTERM_URL") {
            url = Some(v);
        }
        if let Ok(v) = std::env::var("GRAYTERM_TOKEN") {
            token = Some(v);
        }
        Self::from_parts(url, token)
    }

    /// Constructs a Config from optional url and token, failing on missing or empty values.
    ///
    /// <purpose-start>
    /// Provides a testable entry point for validation logic, separate from
    /// the file/env I/O in Config::load().
    /// [initial-implementation.md]
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `url`: Optional Graylog base URL.
    /// - `token`: Optional access token.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// Returns Ok(Config) or a descriptive error.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// None.
    /// <side-effects-end>
    pub(crate) fn from_parts(url: Option<String>, token: Option<String>) -> Result<Self> {
        let url =
            url.context("Graylog URL is required — set GRAYTERM_URL or config [graylog].url")?;
        let token = token.context(
            "Graylog token is required — set GRAYTERM_TOKEN or config [graylog].token",
        )?;
        if url.is_empty() {
            bail!("GRAYTERM_URL must not be empty");
        }
        if token.is_empty() {
            bail!("GRAYTERM_TOKEN must not be empty");
        }
        Ok(Config { url, token })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_present_returns_config() {
        let cfg =
            Config::from_parts(Some("https://graylog.example.com".into()), Some("tok".into()))
                .unwrap();
        assert_eq!(cfg.url, "https://graylog.example.com");
        assert_eq!(cfg.token, "tok");
    }

    #[test]
    fn missing_url_returns_descriptive_error() {
        let err = Config::from_parts(None, Some("tok".into())).unwrap_err();
        assert!(err.to_string().contains("URL"), "error was: {err}");
    }

    #[test]
    fn missing_token_returns_descriptive_error() {
        let err = Config::from_parts(Some("https://example.com".into()), None).unwrap_err();
        assert!(err.to_string().contains("token"), "error was: {err}");
    }

    #[test]
    fn empty_url_returns_error() {
        let err = Config::from_parts(Some(String::new()), Some("tok".into())).unwrap_err();
        assert!(err.to_string().contains("empty"), "error was: {err}");
    }

    #[test]
    fn empty_token_returns_error() {
        let err = Config::from_parts(Some("https://example.com".into()), Some(String::new()))
            .unwrap_err();
        assert!(err.to_string().contains("empty"), "error was: {err}");
    }
}
