// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use crate::ArchitectureLintRule;
use crate::declare_variable_severity_lint;
use crate::helpers::lint_helpers::span_lint_and_help;
use cargo_pup_lint_config::{ConfiguredLint, StructMatch, StructRule};
use regex::Regex;
use rustc_hir::{Item, ItemKind, def_id::DefId};
use rustc_lint::{LateContext, LateLintPass, LintStore};
use rustc_session::impl_lint_pass;
use rustc_span::BytePos;

pub struct StructLint {
    name: String,
    matches: StructMatch,
    struct_rules: Vec<StructRule>,
}

fn attribute_matches(regex: &Regex, attribute: &str) -> bool {
    regex
        .find(attribute)
        .is_some_and(|matched| matched.start() == 0 && matched.end() == attribute.len())
}

fn hir_attribute_name(attribute: &rustc_hir::Attribute) -> Option<String> {
    use rustc_hir::attrs::AttributeKind;

    match attribute {
        rustc_hir::Attribute::Unparsed(attribute) => Some(attribute.path.to_string()),
        rustc_hir::Attribute::Parsed(kind) => match kind {
            AttributeKind::Deprecation { .. } => Some("deprecated".into()),
            AttributeKind::Doc(_) | AttributeKind::DocComment { .. } => Some("doc".into()),
            AttributeKind::MustUse { .. } => Some("must_use".into()),
            AttributeKind::NonExhaustive(_) => Some("non_exhaustive".into()),
            AttributeKind::Repr { .. } => Some("repr".into()),
            _ => None,
        },
    }
}

fn matcher_references_trait(matcher: &StructMatch, trait_path: &str) -> bool {
    match matcher {
        StructMatch::ImplementsTrait(pattern) => Regex::new(pattern)
            .map(|regex| regex.is_match(trait_path))
            .unwrap_or(false),
        StructMatch::AndMatches(left, right) | StructMatch::OrMatches(left, right) => {
            matcher_references_trait(left, trait_path)
                || matcher_references_trait(right, trait_path)
        }
        StructMatch::NotMatch(inner) => matcher_references_trait(inner, trait_path),
        StructMatch::Name(_) | StructMatch::HasAttribute(_) => false,
    }
}

fn evaluate_rule_with(rule: &StructRule, evaluate_leaf: &impl Fn(&StructRule) -> bool) -> bool {
    match rule {
        StructRule::And(left, right) => {
            evaluate_rule_with(left, evaluate_leaf) && evaluate_rule_with(right, evaluate_leaf)
        }
        StructRule::Or(left, right) => {
            evaluate_rule_with(left, evaluate_leaf) || evaluate_rule_with(right, evaluate_leaf)
        }
        StructRule::Not(inner) => !evaluate_rule_with(inner, evaluate_leaf),
        leaf => evaluate_leaf(leaf),
    }
}

fn collect_rule_violations<'a>(
    rule: &'a StructRule,
    expected: bool,
    evaluate_leaf: &impl Fn(&StructRule) -> bool,
    violations: &mut Vec<(&'a StructRule, bool)>,
) {
    if evaluate_rule_with(rule, evaluate_leaf) == expected {
        return;
    }

    match rule {
        StructRule::And(left, right) if expected => {
            collect_rule_violations(left, true, evaluate_leaf, violations);
            collect_rule_violations(right, true, evaluate_leaf, violations);
        }
        StructRule::And(left, right) => {
            // Both children are true when !(left && right) fails. Reporting both
            // alternatives makes the available ways to satisfy the rule visible.
            collect_rule_violations(left, false, evaluate_leaf, violations);
            collect_rule_violations(right, false, evaluate_leaf, violations);
        }
        StructRule::Or(left, right) if expected => {
            // Both alternatives failed, so report both leaf requirements.
            collect_rule_violations(left, true, evaluate_leaf, violations);
            collect_rule_violations(right, true, evaluate_leaf, violations);
        }
        StructRule::Or(left, right) => {
            collect_rule_violations(left, false, evaluate_leaf, violations);
            collect_rule_violations(right, false, evaluate_leaf, violations);
        }
        StructRule::Not(inner) => {
            collect_rule_violations(inner, !expected, evaluate_leaf, violations);
        }
        leaf => violations.push((leaf, expected)),
    }
}

impl StructLint {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(config: &ConfiguredLint) -> Box<dyn ArchitectureLintRule + Send> {
        if let ConfiguredLint::Struct(s) = config {
            Box::new(Self {
                name: s.name.clone(),
                matches: s.matches.clone(),
                struct_rules: s.rules.to_vec(),
            })
        } else {
            panic!("Expected a Struct lint configuration")
        }
    }

    fn matches_struct<'tcx>(
        &self,
        ctx: &LateContext<'tcx>,
        item: &'tcx Item<'tcx>,
        crate_name: &str,
        struct_name: &str,
    ) -> bool {
        self.evaluate_struct_match(&self.matches, ctx, item, crate_name, struct_name)
    }

    fn evaluate_struct_match<'tcx>(
        &self,
        matcher: &StructMatch,
        ctx: &LateContext<'tcx>,
        item: &'tcx Item<'tcx>,
        crate_name: &str,
        struct_name: &str,
    ) -> bool {
        match matcher {
            StructMatch::Name(pattern) => {
                // For compatibility, Name patterns beginning with `test_` match the crate name.
                if pattern.starts_with("test_") {
                    self.string_matches_pattern(crate_name, pattern)
                } else {
                    self.string_matches_pattern(struct_name, pattern)
                }
            }
            StructMatch::HasAttribute(pattern) => self.has_matching_attribute(ctx, item, pattern),
            StructMatch::ImplementsTrait(pattern) => {
                self.implements_matching_trait(ctx, item.owner_id.to_def_id(), pattern)
            }
            StructMatch::AndMatches(left, right) => {
                self.evaluate_struct_match(left, ctx, item, crate_name, struct_name)
                    && self.evaluate_struct_match(right, ctx, item, crate_name, struct_name)
            }
            StructMatch::OrMatches(left, right) => {
                self.evaluate_struct_match(left, ctx, item, crate_name, struct_name)
                    || self.evaluate_struct_match(right, ctx, item, crate_name, struct_name)
            }
            StructMatch::NotMatch(inner) => {
                !self.evaluate_struct_match(inner, ctx, item, crate_name, struct_name)
            }
        }
    }

    // Helper to determine if a string matches a pattern (exact match or regex)
    fn string_matches_pattern(&self, string: &str, pattern: &str) -> bool {
        match Regex::new(pattern) {
            Ok(regex) => regex.is_match(string),
            Err(_) => string == pattern,
        }
    }

    fn describe_pattern(&self, pattern: &str) -> &'static str {
        if pattern.contains(['*', '.', '+', '[', '(', '|', '^', '$', '\\']) {
            "pattern"
        } else {
            "name"
        }
    }

    fn has_matching_attribute(
        &self,
        ctx: &LateContext<'_>,
        item: &Item<'_>,
        pattern: &str,
    ) -> bool {
        let Ok(regex) = Regex::new(pattern) else {
            return false;
        };
        ctx.tcx.hir_attrs(item.hir_id()).iter().any(|attribute| {
            hir_attribute_name(attribute)
                .is_some_and(|attribute| attribute_matches(&regex, &attribute))
        })
    }

    fn implements_matching_trait(
        &self,
        ctx: &LateContext<'_>,
        def_id: DefId,
        trait_pattern: &str,
    ) -> bool {
        use crate::helpers::queries;

        let Ok(trait_regex) = Regex::new(trait_pattern) else {
            return false;
        };
        let ty = ctx.tcx.type_of(def_id).skip_binder();

        ctx.tcx.all_traits_including_private().any(|trait_def_id| {
            let full_trait_name =
                queries::get_full_canonical_trait_name_from_def_id(&ctx.tcx, trait_def_id);
            trait_regex.is_match(&full_trait_name)
                && queries::implements_trait(ctx.tcx, ctx.param_env, ty, trait_def_id)
        })
    }
}

declare_variable_severity_lint!(
    pub,
    STRUCT_LINT_MUST_BE_NAMED,
    STRUCT_LINT_MUST_BE_NAMED_DENY,
    STRUCT_LINT_MUST_BE_NAMED_WARN,
    "Struct naming and attribute rules"
);

declare_variable_severity_lint!(
    pub,
    STRUCT_LINT_MUST_NOT_BE_NAMED,
    STRUCT_LINT_MUST_NOT_BE_NAMED_DENY,
    STRUCT_LINT_MUST_NOT_BE_NAMED_WARN,
    "Struct naming and attribute rules"
);

declare_variable_severity_lint!(
    pub,
    STRUCT_LINT_MUST_BE_PRIVATE,
    STRUCT_LINT_MUST_BE_PRIVATE_DENY,
    STRUCT_LINT_MUST_BE_PRIVATE_WARN,
    "Struct must have private visibility"
);

declare_variable_severity_lint!(
    pub,
    STRUCT_LINT_MUST_BE_PUBLIC,
    STRUCT_LINT_MUST_BE_PUBLIC_DENY,
    STRUCT_LINT_MUST_BE_PUBLIC_WARN,
    "Struct must have public visibility"
);

declare_variable_severity_lint!(
    pub,
    STRUCT_LINT_MUST_BE_PUB_CRATE,
    STRUCT_LINT_MUST_BE_PUB_CRATE_DENY,
    STRUCT_LINT_MUST_BE_PUB_CRATE_WARN,
    "Struct must have pub(crate) visibility"
);

declare_variable_severity_lint!(
    pub,
    STRUCT_LINT_IMPLEMENTS_TRAIT,
    STRUCT_LINT_IMPLEMENTS_TRAIT_DENY,
    STRUCT_LINT_IMPLEMENTS_TRAIT_WARN,
    "Struct trait implementation rules"
);

impl_lint_pass!(StructLint => [
    STRUCT_LINT_MUST_BE_NAMED_DENY,
    STRUCT_LINT_MUST_BE_NAMED_WARN,
    STRUCT_LINT_MUST_NOT_BE_NAMED_DENY,
    STRUCT_LINT_MUST_NOT_BE_NAMED_WARN,
    STRUCT_LINT_MUST_BE_PRIVATE_DENY,
    STRUCT_LINT_MUST_BE_PRIVATE_WARN,
    STRUCT_LINT_MUST_BE_PUBLIC_DENY,
    STRUCT_LINT_MUST_BE_PUBLIC_WARN,
    STRUCT_LINT_MUST_BE_PUB_CRATE_DENY,
    STRUCT_LINT_MUST_BE_PUB_CRATE_WARN,
    STRUCT_LINT_IMPLEMENTS_TRAIT_DENY,
    STRUCT_LINT_IMPLEMENTS_TRAIT_WARN
]);

impl ArchitectureLintRule for StructLint {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn applies_to_module(&self, _namespace: &str) -> bool {
        false
    }

    fn applies_to_trait(&self, trait_path: &str) -> bool {
        matcher_references_trait(&self.matches, trait_path)
    }

    fn register_late_pass(&self, lint_store: &mut LintStore) {
        let name = self.name.clone();
        let matches = self.matches.clone();
        let struct_rules = self.struct_rules.clone();

        lint_store.register_late_pass(move |_| {
            Box::new(StructLint {
                name: name.clone(),
                matches: matches.clone(),
                struct_rules: struct_rules.clone(),
            })
        });
    }
}

impl<'tcx> LateLintPass<'tcx> for StructLint {
    fn check_item(&mut self, ctx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        // We only care about struct items
        if let ItemKind::Struct(..) = item.kind {
            let item_name = ctx
                .tcx
                .item_name(item.owner_id.def_id.to_def_id())
                .to_string();
            let crate_name = ctx
                .tcx
                .crate_name(rustc_hir::def_id::LOCAL_CRATE)
                .to_string();

            if !self.matches_struct(ctx, item, &crate_name, &item_name) {
                return;
            }

            let def_id = item.owner_id.def_id.to_def_id();

            // Create a span that only covers the struct definition line
            // This includes "pub struct Name {" but not the struct fields or closing brace
            let definition_span = {
                let span = item.span;
                // Check if the struct is public by checking if ident_span is different from vis_span
                let is_pub = !item.vis_span.is_empty();
                let prefix_len = if is_pub { 11 } else { 7 }; // "pub struct " or "struct "

                // Create a span from start to just after the struct name and opening brace
                let end_pos = span.lo() + BytePos((prefix_len + item_name.len() + 2) as u32); // +2 for " {"
                span.with_hi(end_pos)
            };

            // Get visibility of the struct
            let struct_visibility = ctx.tcx.visibility(def_id);
            let is_public = struct_visibility == rustc_middle::ty::Visibility::Public;

            // Check if there's a visibility keyword (pub, pub(crate), pub(super), etc.)
            let has_visibility_keyword = !item.vis_span.is_empty();

            // Check if visibility is pub(crate):
            // - Must have a visibility keyword (not inherited/private)
            // - Must be restricted to the crate root
            let is_pub_crate = match struct_visibility {
                rustc_middle::ty::Visibility::Restricted(restricted_to) => {
                    has_visibility_keyword && restricted_to.is_crate_root()
                }
                _ => false,
            };

            // Truly private means no visibility keyword at all (inherited visibility)
            let is_private = !has_visibility_keyword;

            let evaluate_leaf = |rule: &StructRule| match rule {
                StructRule::MustBeNamed(pattern, _) => {
                    self.string_matches_pattern(&item_name, pattern)
                }
                StructRule::MustNotBeNamed(pattern, _) => {
                    !self.string_matches_pattern(&item_name, pattern)
                }
                StructRule::MustBePrivate(_) => is_private,
                StructRule::MustBePublic(_) => is_public,
                StructRule::MustBePubCrate(_) => is_pub_crate,
                StructRule::ImplementsTrait(pattern, _) => {
                    self.implements_matching_trait(ctx, def_id, pattern)
                }
                StructRule::And(_, _) | StructRule::Or(_, _) | StructRule::Not(_) => {
                    unreachable!("composite rules are evaluated recursively")
                }
            };

            for configured_rule in &self.struct_rules {
                let mut violations = Vec::new();
                collect_rule_violations(configured_rule, true, &evaluate_leaf, &mut violations);

                for (rule, expected) in violations {
                    match rule {
                        StructRule::MustBeNamed(pattern, severity)
                        | StructRule::MustNotBeNamed(pattern, severity) => {
                            let must_match =
                                matches!(rule, StructRule::MustBeNamed(..)) == expected;
                            let pattern_type = self.describe_pattern(pattern);
                            let (lint, message, help) = if must_match {
                                let help = if pattern_type == "pattern" {
                                    format!("Rename this struct to match the pattern '{pattern}'")
                                } else {
                                    format!("Rename this struct to '{pattern}'")
                                };
                                (
                                    STRUCT_LINT_MUST_BE_NAMED::get_by_severity(*severity),
                                    format!(
                                        "Struct must match {pattern_type} '{pattern}', found '{item_name}'"
                                    ),
                                    help,
                                )
                            } else {
                                (
                                    STRUCT_LINT_MUST_NOT_BE_NAMED::get_by_severity(*severity),
                                    format!("Struct must not match {pattern_type} '{pattern}'"),
                                    "Choose a name that doesn't match this pattern".to_string(),
                                )
                            };
                            span_lint_and_help(
                                ctx,
                                lint,
                                self.name().as_str(),
                                definition_span,
                                message,
                                None,
                                help,
                            );
                        }
                        StructRule::MustBePrivate(severity) => {
                            let visibility_desc = if is_public {
                                "pub"
                            } else if is_pub_crate {
                                "pub(crate)"
                            } else {
                                "restricted"
                            };
                            let (message, help) = if expected {
                                (
                                    format!(
                                        "Struct '{item_name}' has {visibility_desc} visibility, but must be private"
                                    ),
                                    "Remove the visibility modifier",
                                )
                            } else {
                                (
                                    format!("Struct '{item_name}' must not be private"),
                                    "Add an explicit visibility modifier",
                                )
                            };
                            span_lint_and_help(
                                ctx,
                                STRUCT_LINT_MUST_BE_PRIVATE::get_by_severity(*severity),
                                self.name().as_str(),
                                definition_span,
                                message,
                                None,
                                help,
                            );
                        }
                        StructRule::MustBePublic(severity) => {
                            let visibility_desc = if is_pub_crate {
                                "pub(crate)"
                            } else {
                                "private"
                            };
                            let (message, help) = if expected {
                                (
                                    format!(
                                        "Struct '{item_name}' has {visibility_desc} visibility, but must be pub"
                                    ),
                                    "Change the visibility to 'pub'",
                                )
                            } else {
                                (
                                    format!("Struct '{item_name}' must not be pub"),
                                    "Restrict this struct's visibility",
                                )
                            };
                            span_lint_and_help(
                                ctx,
                                STRUCT_LINT_MUST_BE_PUBLIC::get_by_severity(*severity),
                                self.name().as_str(),
                                definition_span,
                                message,
                                None,
                                help,
                            );
                        }
                        StructRule::MustBePubCrate(severity) => {
                            let visibility_desc = if is_public { "pub" } else { "private" };
                            let (message, help) = if expected {
                                (
                                    format!(
                                        "Struct '{item_name}' has {visibility_desc} visibility, but must be pub(crate)"
                                    ),
                                    "Change the visibility to 'pub(crate)'",
                                )
                            } else {
                                (
                                    format!("Struct '{item_name}' must not be pub(crate)"),
                                    "Change this struct's visibility",
                                )
                            };
                            span_lint_and_help(
                                ctx,
                                STRUCT_LINT_MUST_BE_PUB_CRATE::get_by_severity(*severity),
                                self.name().as_str(),
                                definition_span,
                                message,
                                None,
                                help,
                            );
                        }
                        StructRule::ImplementsTrait(pattern, severity) => {
                            let (message, help) = if expected {
                                (
                                    format!(
                                        "Struct '{item_name}' must implement trait matching '{pattern}'"
                                    ),
                                    "Implement the required trait for this struct",
                                )
                            } else {
                                (
                                    format!(
                                        "Struct '{item_name}' must not implement trait matching '{pattern}'"
                                    ),
                                    "Remove the forbidden trait implementation",
                                )
                            };
                            span_lint_and_help(
                                ctx,
                                STRUCT_LINT_IMPLEMENTS_TRAIT::get_by_severity(*severity),
                                self.name().as_str(),
                                definition_span,
                                message,
                                None,
                                help,
                            );
                        }
                        StructRule::And(_, _) | StructRule::Or(_, _) | StructRule::Not(_) => {
                            unreachable!("only leaf rule violations are reported")
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        attribute_matches, collect_rule_violations, evaluate_rule_with, matcher_references_trait,
    };
    use cargo_pup_lint_config::{Severity, StructMatch, StructRule};
    use regex::Regex;

    fn named(pattern: &str) -> StructRule {
        StructRule::MustBeNamed(pattern.to_string(), Severity::Warn)
    }

    fn leaf_value(rule: &StructRule) -> bool {
        match rule {
            StructRule::MustBeNamed(pattern, _) => pattern == "passes",
            _ => unreachable!(),
        }
    }

    #[test]
    fn attribute_patterns_match_the_complete_name() {
        let regex = Regex::new("repr").unwrap();

        assert!(attribute_matches(&regex, "repr"));
        assert!(!attribute_matches(&regex, "other_repr"));
    }

    #[test]
    fn identifies_traits_referenced_by_complex_matchers() {
        let matcher = StructMatch::AndMatches(
            Box::new(StructMatch::Name("Service".into())),
            Box::new(StructMatch::NotMatch(Box::new(
                StructMatch::ImplementsTrait("^crate::RequiredTrait$".into()),
            ))),
        );

        assert!(matcher_references_trait(&matcher, "crate::RequiredTrait"));
        assert!(!matcher_references_trait(&matcher, "crate::OtherTrait"));
    }

    #[test]
    fn evaluates_logical_rule_combinations() {
        let pass = named("passes");
        let fail = named("fails");

        assert!(evaluate_rule_with(
            &StructRule::And(Box::new(pass.clone()), Box::new(pass.clone())),
            &leaf_value,
        ));
        assert!(evaluate_rule_with(
            &StructRule::Or(Box::new(fail.clone()), Box::new(pass.clone())),
            &leaf_value,
        ));
        assert!(evaluate_rule_with(
            &StructRule::Not(Box::new(fail)),
            &leaf_value,
        ));
    }

    #[test]
    fn reports_both_failed_or_alternatives() {
        let rule = StructRule::Or(Box::new(named("first")), Box::new(named("second")));
        let mut violations = Vec::new();

        collect_rule_violations(&rule, true, &leaf_value, &mut violations);

        assert_eq!(violations.len(), 2);
        assert!(violations.iter().all(|(_, expected)| *expected));
    }

    #[test]
    fn inverts_not_rule_expectation() {
        let rule = StructRule::Not(Box::new(named("passes")));
        let mut violations = Vec::new();

        collect_rule_violations(&rule, true, &leaf_value, &mut violations);

        assert_eq!(violations.len(), 1);
        assert!(!violations[0].1);
    }
}
