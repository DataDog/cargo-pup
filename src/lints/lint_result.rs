use ansi_term::Color;
use rustc_span::Span;
use serde::Deserialize;

/// Severity levels for lint results
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Severity {
    Info,
    Warn,
    Error,
}

/// Represents a lint result
#[derive(Debug)]
pub struct LintResult {
    // The name of the lint itself ('namespace', 'function_length', etc.)
    pub lint: String,

    // The name of this configured lint rule (user supplied)
    pub lint_name: String,

    pub span: Span,
    pub message: String,
    pub severity: Severity,
}

/// Result of a lint run
impl LintResult {
    /// Convert the lint result to a user-readable string with file and line information
    pub fn to_string(&self, source_map: &rustc_span::source_map::SourceMap) -> String {
        // Get the file name
        let file_name = source_map
            .span_to_filename(self.span)
            .display(rustc_span::FileNameDisplayPreference::Local)
            .to_string();

        // Get the snippet and line information
        let line_info = source_map.lookup_line(self.span.lo());
        let snippet = source_map.span_to_snippet(self.span).unwrap_or_default();

        match line_info {
            Ok(line_info) => {
                let line_number = line_info.line + 1; // Line numbers are 0-based
                let line_indent = " ".repeat(line_number.to_string().len() + 1);
                format!(
                    "{} [{}::{}]: {}\n{}\n{}|\n{} | {}\n{}| {}\n",
                    self.severity_to_string(),
                    self.lint,
                    self.lint_name,
                    self.message,
                    file_name,
                    line_indent,
                    line_number,
                    snippet,
                    line_indent,
                    "^".repeat(snippet.len()) // TODO - we should make this just highlight the span, not the whole line
                )
            }
            Err(_) => {
                // Fallback if line information is unavailable
                format!(
                    "{}: {} at {:?}: {}",
                    self.severity_to_string(),
                    self.message,
                    self.span,
                    snippet
                )
            }
        }
    }

    /// Converts severity into a user-readable string
    fn severity_to_string(&self) -> String {
        match self.severity {
            Severity::Info => Color::Blue.bold().paint("info").to_string(),
            Severity::Warn => Color::Yellow.bold().paint("warning").to_string(),
            Severity::Error => Color::Red.bold().paint("error").to_string(),
        }
    }
}
