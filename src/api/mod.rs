/// Graylog API client and related types.
///
/// <purpose-start>
/// Groups all Graylog REST API interaction under one module so the rest of
/// the crate only imports from `api::*` rather than reaching into sub-modules
/// directly.
/// [initial-implementation.md]
/// <purpose-end>
pub mod client;
pub mod search;
pub mod streams;

pub use client::GraylogClient;
