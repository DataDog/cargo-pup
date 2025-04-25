use rustc_lint::LintStore;

///
/// One of our lints. These are an abstraction over the top of the
/// core rustc 'LateLintPass' - which does the actual linting in the
/// compilation process.
///
/// They add:
/// * A name, which is used in the diagnostics to refer back to the configuration
///   item.
/// * The ability to check if certain items are targeted by our lint - for instance,
///   namespaces - so that we can print a diagnostic tree of what our rules are actually
///   doing.
pub trait ArchitectureLintRule: Sync + Send {
    ///
    /// Returns the name of the lint rule. This is the name specified
    /// in pup.yaml
    ///
    fn name(&self) -> String;

    ///
    /// Returns true if the given lint applies to the particular module, false
    /// otherwise. A lint only applies to a module if it is directly constraining
    /// it in some fashion - not if it applies to _some element_ in the module.
    /// In practice, this means configured lints that have a property like "module_name:".
    ///
    /// This is used to annotate the diagnostic tree of `cargo pup print-namespaces` to indicate
    /// which rules applies to a particular
    fn applies_to_module(&self, namespace: &str) -> bool;
    
    ///
    /// Returns true if the given lint applies to the particular trait, false
    /// otherwise. A lint only applies to a trait if it is directly constraining
    /// it in some fashion. In practice, this means configured lints that specifically
    /// target trait definitions or implementations.
    ///
    /// This is used to annotate trait information in `cargo pup print-traits` to indicate
    /// which rules apply to a particular trait
    fn applies_to_trait(&self, _trait_path: &str) -> bool {
        // Default implementation: no lints apply to traits by default
        false
    }
    
    fn register_late_pass(&self, _lint_store: &mut LintStore) {}
}
