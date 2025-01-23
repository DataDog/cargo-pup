use std::{io::Write, sync::Once};
use tempfile::TempDir;

use rustc_driver::RunCompiler;
use rustc_session::{config::ErrorOutputType, EarlyDiagCtxt};

use crate::lints::{ArchitectureLintCollection, ArchitectureLintRule};

static INIT: Once = Once::new();

/// Confirm that the expected set of lint results was returned. If it wasn't, print all the
/// lint results out to stderr.
pub fn assert_lint_results(expected_count: usize, collection: &ArchitectureLintCollection) {
    if collection.lint_results().len() != expected_count {
        eprintln!(
            "Expected {} lint results, got {}. Dumping results:\n {}",
            expected_count,
            collection.lint_results().len(),
            collection.to_string()
        );
        assert_eq!(expected_count, collection.lint_results().len());
    }
}

pub fn lints_for_code(
    code: &str,
    lint: impl ArchitectureLintRule + Send + 'static,
) -> ArchitectureLintCollection {
    // Initialize the Rust compiler's environment logger once
    INIT.call_once(|| {
        let early_dcx = EarlyDiagCtxt::new(ErrorOutputType::default());
        rustc_driver::init_rustc_env_logger(&early_dcx);
    });

    // Create a unique temporary directory for this test
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.path();

    // Write the code to a file named "test.rs" in the temporary directory
    let mod_file_path = temp_dir_path.join("test.rs");
    let crate_root = temp_dir_path.join("mod.rs");

    // Create and write the module declaration to "mod.rs"
    std::fs::write(&crate_root, "mod test;").expect("Failed to write mod.rs file");

    // Create and write the test code to "test.rs"
    let mut mod_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&mod_file_path)
        .expect("Failed to create or truncate test.rs file");

    mod_file
        .write_all(code.as_bytes())
        .expect("Failed to write code to test.rs file");

    println!("test file: {:?}", mod_file);

    // Prepare the arguments for the Rust compiler
    let args: Vec<String> = vec![
        "rustc".into(),
        mod_file_path.to_str().unwrap().into(),
        "--crate-name".into(),
        "test".into(),
        "--crate-type".into(),
        "lib".into(),
        "--out-dir".into(),
        temp_dir_path.to_str().unwrap().into(),
    ];

    // Box up our lints
    let mut callbacks = ArchitectureLintCollection::new(vec![Box::new(lint)]);

    // Run the compiler
    RunCompiler::new(&args, &mut callbacks).run();

    callbacks
}
