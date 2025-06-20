// This product includes software developed at Datadog (https://www.datadoghq.com/) Copyright 2024 Datadog, Inc.

use cargo_metadata::MetadataCommand;
use std::path::PathBuf;

/// Find pup.ron in workspace root using cargo metadata
pub fn find_workspace_pup_ron() -> Option<PathBuf> {
    let metadata = MetadataCommand::new().no_deps().exec().ok()?;
    let pup_ron = metadata.workspace_root.join("pup.ron");
    if pup_ron.exists() {
        Some(pup_ron.into_std_path_buf())
    } else {
        None
    }
}
