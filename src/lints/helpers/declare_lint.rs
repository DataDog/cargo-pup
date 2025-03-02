///
/// Lets us create two lints in the fashion of declare_lint!,
/// but with a variant for Deny and a variant for Warn, using
/// the same lint name.
///
/// This lets us do dynamic lint level selection at runtime
/// based on the user's configuration.
///
#[macro_export]
macro_rules! declare_variable_severity_lint {
    ($(#[$attr:meta])* $vis: vis, $NAME: ident, $NAME_DENY: ident, $NAME_WARN: ident, $desc: expr) => (
        // Deny severity: Directly define the lint with the Deny severity.
        $(#[$attr])*
        $vis static $NAME_DENY: &rustc_session::lint::Lint = &rustc_session::lint::Lint {
            name: stringify!($NAME),  // The name for the lint (denoted by `$NAME`)
            default_level: rustc_session::lint::Level::Deny,  // Set severity to Deny
            desc: $desc,  // Use the description passed into the macro
            is_externally_loaded: false,
            // Add any additional fields as needed
            ..rustc_session::lint::Lint::default_fields_for_macro()  // Default values for other fields
        };

        // Warn severity: Directly define the lint with the Warn severity.
        $(#[$attr])*
        $vis static $NAME_WARN: &rustc_session::lint::Lint = &rustc_session::lint::Lint {
            name: stringify!($NAME),  // The name for the lint (denoted by `$NAME`)
            default_level: rustc_session::lint::Level::Warn,  // Set severity to Warn
            desc: $desc,  // Use the description passed into the macro
            is_externally_loaded: false,
            // Add any additional fields as needed
            ..rustc_session::lint::Lint::default_fields_for_macro()  // Default values for other fields
        };

        fn get_lint(severity: Severity) -> &'static Lint {
            match severity {
                Severity::Warn => $NAME_WARN,
                Severity::Error => $NAME_DENY,
            }
        }
    );
}
