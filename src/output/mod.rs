/// Output format for grayterm commands.
///
/// <purpose-start>
/// Enumerates the rendering formats accepted by the Graylog Search Scripting
/// API. The value maps directly to the HTTP Accept header. Doubles as the
/// clap ValueEnum for all --format flags, keeping the CLI and HTTP layers in
/// sync without a conversion step.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <side-effects-start>
/// None.
/// <side-effects-end>
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Default)]
pub enum OutputFormat {
    /// Human-readable plain text rendered by the server.
    #[default]
    Text,
    /// Comma-separated values.
    Csv,
    /// Raw JSON.
    Json,
}

impl OutputFormat {
    /// Returns the HTTP Accept header value for this format.
    ///
    /// <purpose-start>
    /// Maps each format variant to the MIME type expected by the Graylog
    /// Search Scripting API so the server renders results in the requested
    /// format before they reach grayterm.
    /// [initial-implementation.md]
    /// <purpose-end>
    ///
    /// <outputs-start>
    /// Returns a static string suitable for use as an Accept header value.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// None.
    /// <side-effects-end>
    pub fn accept_header(self) -> &'static str {
        match self {
            OutputFormat::Text => "text/plain",
            OutputFormat::Csv => "text/csv",
            OutputFormat::Json => "application/json",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_maps_to_text_plain() {
        assert_eq!(OutputFormat::Text.accept_header(), "text/plain");
    }

    #[test]
    fn csv_maps_to_text_csv() {
        assert_eq!(OutputFormat::Csv.accept_header(), "text/csv");
    }

    #[test]
    fn json_maps_to_application_json() {
        assert_eq!(OutputFormat::Json.accept_header(), "application/json");
    }

    #[test]
    fn default_is_text() {
        assert_eq!(OutputFormat::default(), OutputFormat::Text);
    }
}
