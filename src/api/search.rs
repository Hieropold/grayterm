use anyhow::{bail, Result};
use serde::Serialize;
use serde_json::Value;

/// Parses a human-readable duration string into a number of seconds.
///
/// <purpose-start>
/// Converts --last CLI arguments such as "1h", "30m", "2d" into the u64
/// seconds value required by the Graylog relative timerange parameter.
/// A dedicated parser avoids requiring users to specify raw seconds.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <inputs-start>
/// - `s`: A string consisting of a positive integer followed by one of
///   `s` (seconds), `m` (minutes), `h` (hours), or `d` (days).
/// <inputs-end>
///
/// <outputs-start>
/// Returns the duration as u64 seconds, or an error describing the problem.
/// <outputs-end>
///
/// <side-effects-start>
/// None.
/// <side-effects-end>
pub fn parse_duration(s: &str) -> Result<u64> {
    if s.is_empty() {
        bail!("duration must not be empty");
    }
    let (num_str, unit) = s.split_at(s.len() - 1);
    let n: u64 = num_str.parse().map_err(|_| {
        anyhow::anyhow!("invalid duration '{s}': expected a positive integer followed by s/m/h/d")
    })?;
    let multiplier = match unit {
        "s" => 1u64,
        "m" => 60,
        "h" => 3_600,
        "d" => 86_400,
        other => bail!("invalid duration unit '{other}' in '{s}': must be s, m, h, or d"),
    };
    Ok(n * multiplier)
}

/// Timerange for a Graylog search query.
///
/// <purpose-start>
/// Encodes the two timerange types supported by grayterm v1. Serializes
/// directly to the Graylog Search Scripting API JSON format so no manual
/// JSON construction is needed at call sites.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <outputs-start>
/// Relative serializes as `{"type":"relative","range":N}`.
/// Absolute serializes as `{"type":"absolute","from":"…","to":"…"}`.
/// <outputs-end>
///
/// <side-effects-start>
/// None.
/// <side-effects-end>
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TimeRange {
    /// Last N seconds relative to now.
    Relative { range: u64 },
    /// Explicit start and end in RFC3339 format.
    Absolute { from: String, to: String },
}

/// Parameters for a Graylog message search.
///
/// <purpose-start>
/// Collects all user-facing search options in one place so the HTTP client
/// only needs a single SearchRequest argument. Optional fields that are None
/// or empty are omitted from the serialised body.
/// [initial-implementation.md]
/// <purpose-end>
///
/// <side-effects-start>
/// None.
/// <side-effects-end>
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// Lucene query string (e.g. `"level:ERROR"`).
    pub query: String,
    /// Stream IDs to scope the search. Empty means all streams.
    pub streams: Vec<String>,
    /// Message fields to include in results.
    pub fields: Vec<String>,
    /// Time window for the search.
    pub timerange: TimeRange,
    /// Maximum number of results (`size` in the API body).
    pub size: Option<u64>,
    /// Field to sort results by.
    pub sort: Option<String>,
    /// Sort direction (`"asc"` or `"desc"`). None when sort is not set.
    pub sort_order: Option<String>,
}

/// Internal serde struct for the POST /api/search/messages body.
///
/// <purpose-start>
/// Separates serialisation concerns from SearchRequest so optional fields
/// can use `skip_serializing_if` cleanly without cluttering the public API.
/// [initial-implementation.md]
/// <purpose-end>
#[derive(Serialize)]
struct SearchBody<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    streams: &'a Vec<String>,
    fields: &'a Vec<String>,
    timerange: &'a TimeRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort_order: Option<&'a str>,
}

impl SearchRequest {
    /// Serialises this request into a JSON value for POST /api/search/messages.
    ///
    /// <purpose-start>
    /// Builds the minimal valid body for the Graylog Search Scripting API,
    /// omitting optional keys that were not supplied so the server applies its
    /// own defaults rather than receiving nulls.
    /// [initial-implementation.md]
    /// <purpose-end>
    ///
    /// <outputs-start>
    /// Returns a serde_json::Value representing the complete request body.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// None.
    /// <side-effects-end>
    pub fn to_body(&self) -> Value {
        let body = SearchBody {
            query: &self.query,
            streams: &self.streams,
            fields: &self.fields,
            timerange: &self.timerange,
            size: self.size,
            sort: self.sort.as_deref(),
            sort_order: self.sort_order.as_deref(),
        };
        serde_json::to_value(&body).expect("SearchBody serialisation is infallible")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── Duration parsing ──────────────────────────────────────────────────────

    #[test]
    fn parse_1h_returns_3600() {
        assert_eq!(parse_duration("1h").unwrap(), 3_600);
    }

    #[test]
    fn parse_30m_returns_1800() {
        assert_eq!(parse_duration("30m").unwrap(), 1_800);
    }

    #[test]
    fn parse_2d_returns_172800() {
        assert_eq!(parse_duration("2d").unwrap(), 172_800);
    }

    #[test]
    fn parse_45s_returns_45() {
        assert_eq!(parse_duration("45s").unwrap(), 45);
    }

    #[test]
    fn parse_invalid_unit_returns_error() {
        let err = parse_duration("1x").unwrap_err();
        assert!(err.to_string().contains("unit"), "error was: {err}");
    }

    #[test]
    fn parse_non_numeric_value_returns_error() {
        let err = parse_duration("abch").unwrap_err();
        assert!(err.to_string().contains("invalid duration"), "error was: {err}");
    }

    #[test]
    fn parse_empty_string_returns_error() {
        assert!(parse_duration("").is_err());
    }

    // ── TimeRange serialisation ───────────────────────────────────────────────

    #[test]
    fn relative_timerange_serialises_correctly() {
        let tr = TimeRange::Relative { range: 3_600 };
        let v = serde_json::to_value(&tr).unwrap();
        assert_eq!(v, json!({"type": "relative", "range": 3600}));
    }

    #[test]
    fn absolute_timerange_serialises_correctly() {
        let tr = TimeRange::Absolute {
            from: "2026-06-12T00:00:00Z".into(),
            to: "2026-06-12T01:00:00Z".into(),
        };
        let v = serde_json::to_value(&tr).unwrap();
        assert_eq!(
            v,
            json!({"type": "absolute", "from": "2026-06-12T00:00:00Z", "to": "2026-06-12T01:00:00Z"})
        );
    }

    // ── SearchRequest body builder ────────────────────────────────────────────

    fn default_req() -> SearchRequest {
        SearchRequest {
            query: "*".into(),
            streams: vec![],
            fields: vec!["timestamp".into(), "source".into(), "message".into()],
            timerange: TimeRange::Relative { range: 3_600 },
            size: None,
            sort: None,
            sort_order: None,
        }
    }

    #[test]
    fn body_includes_query_and_fields() {
        let body = default_req().to_body();
        assert_eq!(body["query"], "*");
        assert_eq!(body["fields"], json!(["timestamp", "source", "message"]));
    }

    #[test]
    fn body_omits_streams_key_when_empty() {
        let body = default_req().to_body();
        assert!(!body.as_object().unwrap().contains_key("streams"));
    }

    #[test]
    fn body_includes_streams_when_set() {
        let mut req = default_req();
        req.streams = vec!["abc123".into()];
        let body = req.to_body();
        assert_eq!(body["streams"], json!(["abc123"]));
    }

    #[test]
    fn body_includes_relative_timerange() {
        let body = default_req().to_body();
        assert_eq!(body["timerange"], json!({"type": "relative", "range": 3600}));
    }

    #[test]
    fn body_includes_absolute_timerange() {
        let mut req = default_req();
        req.timerange = TimeRange::Absolute {
            from: "2026-06-12T00:00:00Z".into(),
            to: "2026-06-12T01:00:00Z".into(),
        };
        let body = req.to_body();
        assert_eq!(
            body["timerange"],
            json!({"type": "absolute", "from": "2026-06-12T00:00:00Z", "to": "2026-06-12T01:00:00Z"})
        );
    }

    #[test]
    fn body_omits_size_when_none() {
        assert!(!default_req().to_body().as_object().unwrap().contains_key("size"));
    }

    #[test]
    fn body_includes_size_when_set() {
        let mut req = default_req();
        req.size = Some(100);
        assert_eq!(req.to_body()["size"], 100);
    }

    #[test]
    fn body_omits_sort_keys_when_none() {
        let body = default_req().to_body();
        let obj = body.as_object().unwrap();
        assert!(!obj.contains_key("sort"));
        assert!(!obj.contains_key("sort_order"));
    }

    #[test]
    fn body_includes_sort_keys_when_set() {
        let mut req = default_req();
        req.sort = Some("timestamp".into());
        req.sort_order = Some("asc".into());
        let body = req.to_body();
        assert_eq!(body["sort"], "timestamp");
        assert_eq!(body["sort_order"], "asc");
    }
}
