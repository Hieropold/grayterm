/// grayterm library crate.
///
/// <purpose-start>
/// Exposes all application modules as a library target so integration tests
/// in tests/ can import types without going through the binary entry point.
/// The binary (main.rs) then depends on this same library.
/// [initial-implementation.md]
/// <purpose-end>
pub mod api;
pub mod cli;
pub mod config;
pub mod error;
pub mod output;
