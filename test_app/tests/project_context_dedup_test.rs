// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use cargo_pup_common::project_context::{ModuleInfo, ProjectContext, TraitInfo};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn merged_workspace_context_deduplicates_modules_and_traits() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let context_dir = std::env::temp_dir().join(format!("cargo-pup-context-{unique}"));

    let mut first = ProjectContext::with_base_dir(&context_dir);
    first.module_root = "first_crate".into();
    first.modules = vec![ModuleInfo {
        name: "shared::module".into(),
        applicable_lints: vec!["first_lint".into()],
    }];
    first.traits = vec![TraitInfo {
        name: "shared::Trait".into(),
        implementors: vec!["FirstType".into()],
        applicable_lints: vec!["first_lint".into()],
    }];

    let mut second = ProjectContext::with_base_dir(&context_dir);
    second.module_root = "second_crate".into();
    second.modules = vec![ModuleInfo {
        name: "shared::module".into(),
        applicable_lints: vec!["second_lint".into()],
    }];
    second.traits = vec![TraitInfo {
        name: "shared::Trait".into(),
        implementors: vec!["SecondType".into()],
        applicable_lints: vec!["second_lint".into()],
    }];

    first.serialize_to_file().unwrap();
    second.serialize_to_file().unwrap();

    let (merged, crate_names) =
        ProjectContext::load_all_contexts_from_dir(&context_dir).unwrap();

    assert_eq!(crate_names.len(), 2);
    assert_eq!(merged.modules.len(), 1);
    assert_eq!(
        merged.modules[0].applicable_lints,
        vec!["first_lint", "second_lint"]
    );
    assert_eq!(merged.traits.len(), 1);
    assert_eq!(
        merged.traits[0].implementors,
        vec!["FirstType", "SecondType"]
    );
    assert_eq!(
        merged.traits[0].applicable_lints,
        vec!["first_lint", "second_lint"]
    );

    std::fs::remove_dir_all(context_dir).unwrap();
}
