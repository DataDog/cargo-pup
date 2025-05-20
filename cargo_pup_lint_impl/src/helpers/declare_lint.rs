/// New version of declare_variable_severity_lint that uses a struct-based approach.
/// 
/// This provides a more flexible way to create lints with variable severity,
/// allowing multiple lints to be defined in the same file without naming conflicts.
#[macro_export]
macro_rules! declare_variable_severity_lint_new {
    ($(#[$attr:meta])* $vis: vis, $NAME: ident, $NAME_DENY: ident, $NAME_WARN: ident, $desc: expr) => (
        // Deny severity: Directly define the lint with the Deny severity.
        $(#[$attr])*
        $vis static $NAME_DENY: &rustc_session::lint::Lint = &rustc_session::lint::Lint {
            name: stringify!($NAME),  // The name for the lint (denoted by `$NAME`)
            default_level: rustc_session::lint::Level::Deny,  // Set severity to Deny
            desc: $desc,  // Use the description passed into the macro
            is_externally_loaded: false,
            ..rustc_session::lint::Lint::default_fields_for_macro()  // Default values for other fields
        };

        // Warn severity: Directly define the lint with the Warn severity.
        $(#[$attr])*
        $vis static $NAME_WARN: &rustc_session::lint::Lint = &rustc_session::lint::Lint {
            name: stringify!($NAME),  // The name for the lint (denoted by `$NAME`)
            default_level: rustc_session::lint::Level::Warn,  // Set severity to Warn
            desc: $desc,  // Use the description passed into the macro
            is_externally_loaded: false,
            ..rustc_session::lint::Lint::default_fields_for_macro()  // Default values for other fields
        };

        // Create a wrapper type to allow accessing the lint based on severity
        // Note that we need to allow BIG_CAPS_NAMES for the lint name emitted by rustc to have
        // underscores between segments
        #[allow(non_camel_case_types)]
        $vis struct $NAME;
        
        impl $NAME {
            /// Get the appropriate lint based on severity
            pub fn get_by_severity(severity: cargo_pup_lint_config::Severity) -> &'static rustc_session::lint::Lint {
                match severity {
                    cargo_pup_lint_config::Severity::Warn => $NAME_WARN,
                    cargo_pup_lint_config::Severity::Error => $NAME_DENY,
                }
            }
        }
    );
}
