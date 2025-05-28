// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

//! Cargo-pup wrappers around rustc's diagnostic functions.
//!
//! This module contains code derived from rust-clippy, which is dual-licensed under
//! Apache 2.0 and MIT licenses. Original copyright:
//! Copyright (c) 2014 The Rust Project Developers

use rustc_errors::{DiagMessage, MultiSpan, SubdiagMessage};
use rustc_lint::{Lint, LintContext};
use rustc_span::Span;

/// Emit a lint message with an extra `help` message.
///
/// Use this if you want to provide some general help but
/// can't provide a specific machine applicable suggestion.
///
/// The `help` message can be optionally attached to a `Span`.
pub fn span_lint_and_help<T: LintContext>(
    cx: &T,
    lint: &'static Lint,
    rule_name: &str,
    span: impl Into<MultiSpan>,
    msg: impl Into<DiagMessage>,
    help_span: Option<Span>,
    help: impl Into<SubdiagMessage>,
) {
    cx.span_lint(lint, span, |diag| {
        diag.primary_message(msg);
        if let Some(help_span) = help_span {
            diag.span_help(help_span, help.into());
        } else {
            diag.help(help.into());
        }
        diag.note(format!("Applied by cargo-pup rule '{}'.", rule_name));
    });
}
