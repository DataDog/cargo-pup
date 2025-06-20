#![feature(rustc_private)]
// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.
#![warn(rust_2018_idioms, unused_lifetimes)]

use std::env;
use std::path::{Path, PathBuf};
use ui_test::custom_flags::rustfix::RustfixMode;
use ui_test::{Args, Config, error_on_output_conflict, status_emitter};

/// Run the UI tests.
fn main() {
    // Setup the UI test configuration
    let mut args = Args::test().unwrap();
    // Enable --bless to automatically update the expected output
    args.bless |= env::var_os("BLESS").is_some_and(|v| v != "0");

    let target_dir =
        PathBuf::from(env::var_os("CARGO_TARGET_DIR").unwrap_or_else(|| "target".into()));

    let mut config = Config {
        output_conflict_handling: error_on_output_conflict,
        filter_files: env::var("TESTNAME")
            .map(|filters| filters.split(',').map(str::to_string).collect())
            .unwrap_or_default(),
        target: None,
        bless_command: Some("cargo test -- --bless".into()),
        out_dir: target_dir.join("ui_test"),
        ..Config::rustc(Path::new("tests").join("ui"))
    };

    // Set default configurations for test comments
    let defaults = config.comment_defaults.base();
    defaults.exit_status = None.into();
    defaults.require_annotations = Some(ui_test::spanned::Spanned::dummy(true)).into();
    defaults.diagnostic_code_prefix = Some(ui_test::spanned::Spanned::dummy("pup::".into())).into();
    defaults.set_custom("rustfix", RustfixMode::Everything);

    // Configure compiler args
    config.with_args(&args);

    let current_exe_path = env::current_exe().unwrap();
    let deps_path = current_exe_path.parent().unwrap();
    let profile_path = deps_path.parent().unwrap();

    config.program.args.extend(
        [
            "--emit=metadata",
            "-Aunused",
            "-Zui-testing",
            "-Zdeduplicate-diagnostics=no",
            &format!("-Ldependency={}", deps_path.display()),
        ]
        .iter()
        .map(|&s| s.into()),
    );

    // Set the driver to use for tests
    config.program.program = profile_path.join(if cfg!(windows) {
        "pup-driver.exe"
    } else {
        "pup-driver"
    });

    // Run the tests
    let _ = ui_test::run_tests_generic(
        vec![config],
        ui_test::default_file_filter,
        ui_test::default_per_file_config,
        status_emitter::Text::from(args.format),
    );
}
