use rustc_lint::LintStore;

/// Trait for defining architecture-specific lint rules
pub trait ArchitectureLintRule: Sync + Send {

    fn name(&self) -> String;
    fn applies_to_module(&self, namespace: &str) -> bool;
    fn register_late_pass(&self, _lint_store: &mut LintStore) {}
}
