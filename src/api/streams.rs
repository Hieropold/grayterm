use serde::Deserialize;

/// A single Graylog stream as returned by GET /api/streams.
///
/// <purpose-start>
/// Carries the stream fields needed by grayterm: id for scripting/targeting
/// and title for human-readable display in text mode.
/// [initial-implementation.md]
/// <purpose-end>
#[derive(Debug, Clone, Deserialize)]
pub struct Stream {
    /// Graylog internal stream identifier.
    pub id: String,
    /// Human-readable stream name.
    pub title: String,
}

/// Top-level response body of GET /api/streams.
///
/// <purpose-start>
/// Deserialises only the fields grayterm uses from the Graylog streams
/// response, ignoring additional metadata Graylog returns.
/// [initial-implementation.md]
/// <purpose-end>
#[derive(Debug, Deserialize)]
pub struct StreamsResponse {
    /// List of streams visible to the authenticated user.
    pub streams: Vec<Stream>,
}

/// Formats a list of streams as tab-separated id/title lines.
///
/// <purpose-start>
/// Provides a scriptable text representation of streams for the
/// `streams list --format text` output mode. Tab separation makes it
/// easy to process with cut(1), awk, and similar tools.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <inputs-start>
/// - `streams`: Slice of Stream values to format.
/// <inputs-end>
///
/// <outputs-start>
/// Returns a newline-separated string where each line is "<id>\t<title>".
/// Returns an empty string for an empty slice.
/// <outputs-end>
///
/// <side-effects-start>
/// None.
/// <side-effects-end>
pub fn format_streams_text(streams: &[Stream]) -> String {
    streams
        .iter()
        .map(|s| format!("{}\t{}", s.id, s.title))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_multiple_streams_as_tab_separated_lines() {
        let streams = vec![
            Stream { id: "abc123".into(), title: "Errors".into() },
            Stream { id: "def456".into(), title: "Application Logs".into() },
        ];
        assert_eq!(
            format_streams_text(&streams),
            "abc123\tErrors\ndef456\tApplication Logs"
        );
    }

    #[test]
    fn formats_single_stream() {
        let streams = vec![Stream { id: "xyz".into(), title: "My Stream".into() }];
        assert_eq!(format_streams_text(&streams), "xyz\tMy Stream");
    }

    #[test]
    fn formats_empty_list_as_empty_string() {
        assert_eq!(format_streams_text(&[]), "");
    }

    #[test]
    fn deserialises_streams_response_ignoring_extra_fields() {
        let json = r#"{"streams":[{"id":"abc","title":"Test","disabled":false}],"total":1}"#;
        let resp: StreamsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.streams.len(), 1);
        assert_eq!(resp.streams[0].id, "abc");
        assert_eq!(resp.streams[0].title, "Test");
    }
}
