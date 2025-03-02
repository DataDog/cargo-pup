use super::{ArchitectureLintRule, Severity};
use crate::declare_variable_severity_lint;
use crate::lints::helpers::clippy_utils::span_lint_and_help;
use crate::utils::configuration_factory::{LintConfigurationFactory, LintFactory};
use regex::Regex;
use rustc_hir::{Item, ItemKind};
use rustc_lint::{LateContext, LateLintPass, Lint};
use rustc_middle::ty::TyCtxt;
use rustc_session::impl_lint_pass;
use serde::Deserialize;
use crate::lints::helpers::queries::get_full_module_name;

/// Configuration for item type lint rule
#[derive(Debug, Deserialize, Clone)]
pub struct ItemTypeConfiguration {
    pub modules: Vec<String>,
    pub denied_items: Vec<DeniedItemType>,
    pub severity: Severity,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeniedItemType {
    Enum,
    Struct,
    Trait,
    Impl,
    Function,
    Module,
    Static,
    Const,
    Union,
}

/// Item type lint processor
struct ItemTypeLintProcessor {
    name: String,
    rule: ItemTypeConfiguration,
    module_regexps: Vec<Regex>,
}

// Declare lint
declare_variable_severity_lint!(
    pub,
    ITEM_TYPE,
    ITEM_TYPE_DENY,
    ITEM_TYPE_WARN,
    "Item types not allowed in this module"
);
impl_lint_pass!(ItemTypeLintProcessor => [ITEM_TYPE_DENY, ITEM_TYPE_WARN]);

impl ItemTypeLintProcessor {
    pub fn new(name: String, rule: ItemTypeConfiguration) -> Self {
        let module_regexps = rule
            .modules
            .iter()
            .map(|m| Regex::new(m).unwrap_or_else(|_| panic!("Can construct a regexp from {}", m)))
            .collect();

        Self {
            name,
            rule,
            module_regexps,
        }
    }

    fn applies_to_module(&self, tcx: &TyCtxt<'_>, module_def_id: &rustc_hir::OwnerId) -> bool {
        let full_name = get_full_module_name(tcx, module_def_id);
        self.module_regexps
            .iter()
            .any(|r| r.is_match(full_name.as_str()))
    }

    fn is_denied_type(&self, kind: &ItemKind) -> Option<&'static str> {
        match kind {
            ItemKind::Enum(..) if self.rule.denied_items.contains(&DeniedItemType::Enum) => {
                Some("enum")
            }
            ItemKind::Struct(..) if self.rule.denied_items.contains(&DeniedItemType::Struct) => {
                Some("struct")
            }
            ItemKind::Trait(..) if self.rule.denied_items.contains(&DeniedItemType::Trait) => {
                Some("trait")
            }
            ItemKind::Impl(..) if self.rule.denied_items.contains(&DeniedItemType::Impl) => {
                Some("impl")
            }
            ItemKind::Fn { sig: _, generics: _, body: _, has_body: _ } 
                if self.rule.denied_items.contains(&DeniedItemType::Function) => {
                Some("function")
            }
            ItemKind::Mod(..) if self.rule.denied_items.contains(&DeniedItemType::Module) => {
                Some("module")
            }
            ItemKind::Static(..) if self.rule.denied_items.contains(&DeniedItemType::Static) => {
                Some("static")
            }
            ItemKind::Const(..) if self.rule.denied_items.contains(&DeniedItemType::Const) => {
                Some("const")
            }
            ItemKind::Union(..) if self.rule.denied_items.contains(&DeniedItemType::Union) => {
                Some("union")
            }
            _ => None,
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for ItemTypeLintProcessor {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        let module = cx.tcx.hir_get_parent_item(item.hir_id());

        if !self.applies_to_module(&cx.tcx, &module) {
            return;
        }

        if let Some(item_type) = self.is_denied_type(&item.kind) {
            let item_name = cx.tcx.item_name(item.owner_id.def_id.to_def_id());
            span_lint_and_help(
                cx,
                get_lint(self.rule.severity),
                self.name().as_str(),
                item.span,
                format!(
                    "{} '{}' is not allowed in this module",
                    item_type, item_name
                ),
                None,
                "Consider moving this item to a different module",
            );
        }
    }
}

impl ArchitectureLintRule for ItemTypeLintProcessor {
    fn register_late_pass(&self, lint_store: &mut rustc_lint::LintStore) {
        let name = self.name.clone();
        let config = self.rule.clone();
        lint_store.register_late_pass(move |_| {
            Box::new(ItemTypeLintProcessor::new(name.clone(), config.clone()))
        });
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn applies_to_module(&self, module: &str) -> bool {
        self.module_regexps.iter().any(|r| r.is_match(module))
    }
}

/// Factory for creating item type lint processors
pub(crate) struct ItemTypeLintFactory {}

impl ItemTypeLintFactory {
    pub fn new() -> Self {
        ItemTypeLintFactory {}
    }
}

impl LintFactory for ItemTypeLintFactory {
    fn register() {
        LintConfigurationFactory::register_lint_factory("item_type", Self::new());
    }

    fn configure(
        &self,
        rule_name: &str,
        yaml: &serde_yaml::Value,
    ) -> anyhow::Result<Vec<Box<dyn ArchitectureLintRule + Send>>> {
        let raw_config: ItemTypeConfiguration = serde_yaml::from_value(yaml.clone())?;
        Ok(vec![Box::new(ItemTypeLintProcessor::new(
            rule_name.into(),
            raw_config,
        ))])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::configuration_factory::LintConfigurationFactory;

    const CONFIGURATION_YAML: &str = "
test_item_type:
  type: item_type
  modules:
    - \"test_module\"
  denied_items:
    - struct
    - enum
  severity: Warn
";

    #[test]
    pub fn can_load_configuration_via_lint_factory() -> anyhow::Result<()> {
        ItemTypeLintFactory::register();
        let results = LintConfigurationFactory::from_yaml(CONFIGURATION_YAML.to_string())?;
        assert_eq!(results.len(), 1);
        Ok(())
    }
} 