use crate::lints::function_length::FunctionLengthLintFactory;
use crate::lints::namespace::NamespaceUsageLintFactory;
use crate::lints::trait_impl::TraitImplLintFactory;
use crate::utils::configuration_factory::LintFactory;
use ansi_term::Color;
use rustc_driver::Callbacks;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use serde::Deserialize;

pub mod function_length;
pub mod namespace;
mod trait_impl;

/// Trait for defining architecture-specific lint rules
pub trait ArchitectureLintRule {
    fn lint(&self, ctx: TyCtxt<'_>) -> Vec<LintResult>;
}

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

///
/// Collects a set of architecture lints configured
/// and ready to run.
/// Provides an implementation of Callbacks and adapts the
/// rustc compiler expectations to what the lints need.
///
pub struct ArchitectureLintCollection {
    lints: Vec<Box<dyn ArchitectureLintRule + Send>>,
    lint_results: Vec<LintResult>,
    lint_results_text: String,
}

impl ArchitectureLintCollection {
    pub fn new(lints: Vec<Box<dyn ArchitectureLintRule + Send>>) -> ArchitectureLintCollection {
        ArchitectureLintCollection {
            lints,
            lint_results: vec![],
            lint_results_text: String::new(),
        }
    }

    ///
    /// Print out all the lint results to a string
    ///
    fn results_to_text(&self, source_map: &rustc_span::source_map::SourceMap) -> String {
        self.lint_results
            .iter()
            .map(|result| result.to_string(source_map))
            .collect::<Vec<String>>()
            .join("\n")
    }

    ///
    /// Returns all the lint results as a collection
    ///
    pub fn lint_results(&self) -> &Vec<LintResult> {
        &self.lint_results
    }

    pub fn to_string(&self) -> String {
        self.lint_results_text.clone()
    }
}

///
/// Adapt rustc's callbacks mechanism to our lints, collecting
/// lint results as we go.
///
impl Callbacks for ArchitectureLintCollection {
    fn after_expansion(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        tcx: TyCtxt<'_>,
    ) -> rustc_driver::Compilation {
        let lints = &self.lints; // Extract lints to avoid borrowing `self` later
        let lint_results = lints.iter().flat_map(|lint| lint.lint(tcx)); // Collect results
        self.lint_results.extend(lint_results); // Mutate `self.lint_results` after iteration

        let source_map = tcx.sess.source_map();
        self.lint_results_text = self.results_to_text(source_map);

        rustc_driver::Compilation::Continue
    }
}

///
/// Should be called once at startup to register
/// all the lints with the configuration factory.
pub fn register_all_lints() {
    NamespaceUsageLintFactory::register();
    FunctionLengthLintFactory::register();
    TraitImplLintFactory::register();
}
