use rustc_errors::{DiagInner, FluentBundle};
use rustc_errors::{emitter::Emitter, registry::Registry, translation::Translate};
use rustc_span::{Symbol, source_map::SourceMap};
use std::{
    io::Write,
    sync::{Arc, Mutex, Once},
};
use tempfile::TempDir;
use uuid::Uuid;

use rustc_driver::{self, Callbacks};
use rustc_session::{EarlyDiagCtxt, config::ErrorOutputType};

use crate::lints::{ArchitectureLintCollection, ArchitectureLintRule, Severity};
use crate::lints::function_length::FunctionLengthConfiguration;

static INIT: Once = Once::new();

/// Confirm that the expected set of lint results was returned. If it wasn't, print all the
/// lint results out to stderr.
pub fn assert_lint_results(expected_count: usize, diagnostics: &Vec<DiagInner>) {
    if diagnostics.len() != expected_count {
        eprintln!(
            "Expected {} lint results, got {}. Dumping results:",
            expected_count,
            diagnostics.len()
        );

        for (idx, diagnostic) in diagnostics.iter().enumerate() {
            eprintln!("{}, {}", idx, diagnostic_to_string(diagnostic));
        }

        assert_eq!(expected_count, diagnostics.len());
    }
}

fn diagnostic_to_string(diagnostic: &DiagInner) -> String {
    let (message, _) = diagnostic.messages.first().unwrap();
    match message {
        rustc_errors::DiagMessage::Str(cow) => format!("DiagMessage::Str: {}", cow).to_string(),
        rustc_errors::DiagMessage::Translated(cow) => {
            format!("DiagMessage::Translated: {}", cow.to_string())
        }
        rustc_errors::DiagMessage::FluentIdentifier(cow, cow1) => {
            format!("DiagMessage::FluentIdentifier({:?},{:?})", cow, cow1).to_string()
        }
    }
}

pub fn lints_for_code(
    code: &str,
    lint: impl ArchitectureLintRule + Send + Sync + 'static,
) -> Vec<DiagInner> {
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
    let args = generate_compiler_args(temp_dir_path, mod_file_path);

    // Box up our lints
    let lints = ArchitectureLintCollection::new(vec![Box::new(lint)]);
    let mut runner = DiagnosticTestRunner::new(lints);

    // Run the compiler
    rustc_driver::run_compiler(&args, &mut runner);
    runner.diagnostics()
}

fn generate_compiler_args(
    temp_dir_path: &std::path::Path,
    mod_file_path: std::path::PathBuf,
) -> Vec<String> {
    let args: Vec<String> = vec![
        "rustc".into(),
        mod_file_path.to_str().unwrap().into(),
        "--crate-name".into(),
        "test".into(),
        "--crate-type".into(),
        "lib".into(),
        "--edition".into(),
        "2024".into(),
        "--error-format".into(),
        "json".into(),
        "--json".into(),
        "diagnostic-rendered-ansi,artifacts,future-incompat".into(),
        "--check-cfg".into(),
        "cfg(docsrs,test)".into(),
        "--check-cfg".into(),
        "cfg(feature, values(\"rustc_driver\", \"rustc_hir\", \"rustc_interface\", \"rustc_lint\", \"rustc_middle\", \"rustc_session\", \"rustc_span\"))".into(),
        "--out-dir".into(),
        temp_dir_path.to_str().unwrap().into(),
        "-C".into(),
        "embed-bitcode=no".into(),
        "-C".into(),
        "debuginfo=2".into(),
        "-C".into(),
        "split-debuginfo=unpacked".into(),
        "-C".into(),
        format!("incremental={}", temp_dir_path.to_str().unwrap()).into(),
        "--sysroot".into(),
        "/Users/scott.gerring/.rustup/toolchains/nightly-2025-02-10-aarch64-apple-darwin".into(),
        // "--target-dir".into(),
        // "/tmp/puptest".into()
    ];
    args
}


struct DiagnosticTestRunner {
    lint_collection: Option<ArchitectureLintCollection>,
    buffer: Arc<Mutex<Vec<DiagInner>>>,
}

impl DiagnosticTestRunner {
    fn new(lint_collection: ArchitectureLintCollection) -> Self {
        DiagnosticTestRunner {
            lint_collection: Some(lint_collection),
            buffer: Arc::default(),
        }
    }

    fn diagnostics(&self) -> Vec<DiagInner> {
        let mut buffer = self.buffer.lock().unwrap();
        std::mem::take(&mut *buffer)
    }
}

impl Callbacks for DiagnosticTestRunner {
    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        // Dirty and awful and yet totally tolerable for unit tests
        let lints = Box::leak(Box::new(
            self.lint_collection.take().expect("can extract lints"),
        ));

        // TODO
        // When we register lints, and we're running with this runner for a test,
        // we get panics in rust.
        // Fix this! In the meantim, we've just got the test_app testing.
        //
        config.register_lints = Some(Box::new(move |_sess, lint_store| {
            for lint in lints.lints() {
                lint.register_late_pass(lint_store);
            }
        }));

        let diagnostics = self.buffer.clone();

        config.psess_created = Some(Box::new(move |psess| {
            psess.dcx().set_emitter(Box::new(DebugEmitter {
                source_map: psess.clone_source_map(),
                diagnostics,
            }));

            // Make sure we don't cache by adding a random value to the symbols
            let binding = Uuid::new_v4().to_string();
            let uuid = binding.as_str();
            psess
                .env_depinfo
                .get_mut()
                .insert((Symbol::intern("NOCACHE"), Some(Symbol::intern(uuid))));

            // Add our test lint
        }));
    }
}

struct DebugEmitter {
    source_map: Arc<SourceMap>,
    diagnostics: Arc<Mutex<Vec<DiagInner>>>,
}

impl Translate for DebugEmitter {
    fn fluent_bundle(&self) -> Option<&FluentBundle> {
        None
    }

    fn fallback_fluent_bundle(&self) -> &FluentBundle {
        panic!("this emitter should not translate message")
    }
}

impl Emitter for DebugEmitter {
    fn emit_diagnostic(&mut self, diag: DiagInner, _: &Registry) {
        self.diagnostics
            .lock()
            .expect("can unwrap diagnostics")
            .push(diag);
    }

    fn source_map(&self) -> Option<&SourceMap> {
        Some(&self.source_map)
    }
}
