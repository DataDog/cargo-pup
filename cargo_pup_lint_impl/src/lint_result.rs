use serde::Deserialize;

/// Severity levels for lint results
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Severity {
    Warn,
    Error,
}
