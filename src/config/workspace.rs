/// Workspace configuration stored in `scope-workspace.toml`.
///
/// A workspace groups multiple Scope projects (each with its own `.scope/`
/// directory) and allows federated queries across all members.
use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Top-level workspace manifest parsed from `scope-workspace.toml`.
#[derive(Debug, Deserialize)]
pub struct WorkspaceConfig {
    /// The `[workspace]` section.
    pub workspace: WorkspaceSection,
}

/// The `[workspace]` section of `scope-workspace.toml`.
#[derive(Debug, Deserialize)]
pub struct WorkspaceSection {
    /// Human-readable workspace name.
    pub name: String,
    /// Schema version for forward compatibility. Current: 1.
    pub version: Option<u32>,
    /// Shared defaults inherited by all members unless overridden.
    #[serde(default)]
    pub defaults: WorkspaceDefaults,
    /// Member project entries.
    pub members: Vec<WorkspaceMemberEntry>,
}

/// Shared defaults that apply to all workspace members unless overridden.
#[derive(Debug, Default, Deserialize)]
pub struct WorkspaceDefaults {
    /// Default index settings.
    pub index: Option<IndexDefaults>,
    /// Default output settings.
    pub output: Option<OutputDefaults>,
}

/// Default index settings inherited by workspace members.
#[derive(Debug, Deserialize)]
pub struct IndexDefaults {
    /// Glob patterns to ignore during indexing (additive with project-level ignores).
    pub ignore: Option<Vec<String>>,
}

/// Default output settings inherited by workspace members.
#[derive(Debug, Deserialize)]
pub struct OutputDefaults {
    /// Maximum number of refs to display before truncating.
    pub max_refs: Option<usize>,
    /// Maximum depth for impact analysis traversal.
    pub max_impact_depth: Option<usize>,
}

/// A single member entry in the workspace manifest.
#[derive(Debug, Deserialize)]
pub struct WorkspaceMemberEntry {
    /// Path relative to the workspace root.
    pub path: String,
    /// Optional human-readable name. Defaults to directory basename.
    pub name: Option<String>,
}

impl WorkspaceConfig {
    /// Load and parse a `scope-workspace.toml` from the given path.
    ///
    /// The path should point to the `scope-workspace.toml` file itself.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let config: WorkspaceConfig =
            toml::from_str(&content).with_context(|| "Failed to parse scope-workspace.toml")?;

        // Validate schema version
        if let Some(version) = config.workspace.version {
            if version > 1 {
                bail!(
                    "Unsupported workspace version {}. This version of Scope supports version 1.",
                    version
                );
            }
        }

        Ok(config)
    }

    /// Resolve the display name for a member entry.
    ///
    /// Returns the explicit `name` if set, otherwise the last component
    /// of the `path` (directory basename).
    pub fn resolve_member_name(entry: &WorkspaceMemberEntry) -> String {
        if let Some(ref name) = entry.name {
            return name.clone();
        }
        Path::new(&entry.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Validate the workspace manifest against its root directory.
    ///
    /// Checks:
    /// - All member paths exist on disk.
    /// - All member paths are subdirectories of the workspace root.
    /// - No overlapping member paths (one member is a prefix of another).
    /// - No duplicate member names.
    pub fn validate(&self, workspace_root: &Path) -> Result<()> {
        let canonical_root = workspace_root.canonicalize().with_context(|| {
            format!(
                "Cannot canonicalize workspace root: {}",
                workspace_root.display()
            )
        })?;

        let mut seen_names: HashSet<String> = HashSet::new();
        let mut canonical_paths: Vec<(String, PathBuf)> = Vec::new();

        for entry in &self.workspace.members {
            let name = Self::resolve_member_name(entry);

            // Check unique names
            if !seen_names.insert(name.clone()) {
                bail!("Duplicate member name '{}'", name);
            }

            // Resolve and check existence
            let member_path = workspace_root.join(&entry.path);
            if !member_path.exists() {
                bail!("Member path '{}' does not exist", entry.path);
            }

            // Canonicalize and check it's under workspace root
            let canonical_member = member_path.canonicalize().with_context(|| {
                format!("Cannot canonicalize member path: {}", member_path.display())
            })?;

            if !canonical_member.starts_with(&canonical_root) {
                bail!(
                    "Member '{}' is outside workspace root. \
                     Set allow_external = true to permit this.",
                    name
                );
            }

            canonical_paths.push((name, canonical_member));
        }

        // Check no overlapping paths (one is prefix of another)
        for i in 0..canonical_paths.len() {
            for j in (i + 1)..canonical_paths.len() {
                let (ref name_a, ref path_a) = canonical_paths[i];
                let (ref name_b, ref path_b) = canonical_paths[j];

                if path_a.starts_with(path_b) || path_b.starts_with(path_a) {
                    bail!(
                        "Member '{}' path overlaps with member '{}'. \
                         Each member must have a distinct, non-overlapping directory.",
                        name_a,
                        name_b
                    );
                }
            }
        }

        Ok(())
    }

    /// Generate a TOML string for a workspace manifest.
    ///
    /// Used by `scope workspace init` to write the initial file.
    pub fn generate_toml(name: &str, members: &[(String, String)]) -> String {
        let mut toml = String::new();
        toml.push_str(
            "# scope-workspace.toml — workspace manifest for multi-project Scope queries.\n",
        );
        toml.push_str("# Place this file at the root of your workspace.\n\n");
        toml.push_str("[workspace]\n");
        toml.push_str(&format!("name = \"{name}\"\n"));
        toml.push_str("version = 1\n\n");

        for (member_path, member_name) in members {
            toml.push_str("[[workspace.members]]\n");
            // Always use forward slashes in the manifest
            let normalized = member_path.replace('\\', "/");
            toml.push_str(&format!("path = \"{normalized}\"\n"));
            toml.push_str(&format!("name = \"{member_name}\"\n\n"));
        }

        toml
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper: create a temp workspace with the given member directories.
    fn setup_workspace(member_dirs: &[&str]) -> TempDir {
        let dir = TempDir::new().unwrap();
        for member in member_dirs {
            let member_path = dir.path().join(member);
            std::fs::create_dir_all(&member_path).unwrap();
        }
        dir
    }

    #[test]
    fn parse_valid_workspace_toml() {
        let toml_str = r#"
            [workspace]
            name = "my-workspace"
            version = 1

            [[workspace.members]]
            path = "services/api"
            name = "api"

            [[workspace.members]]
            path = "services/worker"
        "#;

        let config: WorkspaceConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.workspace.name, "my-workspace");
        assert_eq!(config.workspace.version, Some(1));
        assert_eq!(config.workspace.members.len(), 2);
        assert_eq!(config.workspace.members[0].path, "services/api");
        assert_eq!(config.workspace.members[0].name, Some("api".to_string()));
        assert_eq!(config.workspace.members[1].path, "services/worker");
        assert_eq!(config.workspace.members[1].name, None);
    }

    #[test]
    fn parse_workspace_with_defaults() {
        let toml_str = r#"
            [workspace]
            name = "test"

            [workspace.defaults.index]
            ignore = ["node_modules", "dist"]

            [workspace.defaults.output]
            max_refs = 30
            max_impact_depth = 5

            [[workspace.members]]
            path = "api"
        "#;

        let config: WorkspaceConfig = toml::from_str(toml_str).unwrap();
        let index = config.workspace.defaults.index.unwrap();
        assert_eq!(index.ignore.unwrap(), vec!["node_modules", "dist"]);
        let output = config.workspace.defaults.output.unwrap();
        assert_eq!(output.max_refs, Some(30));
        assert_eq!(output.max_impact_depth, Some(5));
    }

    #[test]
    fn member_name_defaults_to_directory_basename() {
        let entry = WorkspaceMemberEntry {
            path: "services/api".to_string(),
            name: None,
        };
        assert_eq!(WorkspaceConfig::resolve_member_name(&entry), "api");
    }

    #[test]
    fn member_name_uses_explicit_name() {
        let entry = WorkspaceMemberEntry {
            path: "services/api-v2".to_string(),
            name: Some("api".to_string()),
        };
        assert_eq!(WorkspaceConfig::resolve_member_name(&entry), "api");
    }

    #[test]
    fn validate_rejects_missing_member_path() {
        let dir = setup_workspace(&[]);

        let toml_str = r#"
            [workspace]
            name = "test"

            [[workspace.members]]
            path = "does-not-exist"
        "#;

        let config: WorkspaceConfig = toml::from_str(toml_str).unwrap();
        let err = config.validate(dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("does not exist"),
            "Expected 'does not exist' error, got: {}",
            err
        );
    }

    #[test]
    fn validate_rejects_duplicate_member_names() {
        let dir = setup_workspace(&["a", "b"]);

        let toml_str = r#"
            [workspace]
            name = "test"

            [[workspace.members]]
            path = "a"
            name = "shared"

            [[workspace.members]]
            path = "b"
            name = "shared"
        "#;

        let config: WorkspaceConfig = toml::from_str(toml_str).unwrap();
        let err = config.validate(dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("Duplicate member name"),
            "Expected duplicate name error, got: {}",
            err
        );
    }

    #[test]
    fn validate_rejects_overlapping_paths() {
        let dir = setup_workspace(&["parent", "parent/child"]);

        let toml_str = r#"
            [workspace]
            name = "test"

            [[workspace.members]]
            path = "parent"

            [[workspace.members]]
            path = "parent/child"
        "#;

        let config: WorkspaceConfig = toml::from_str(toml_str).unwrap();
        let err = config.validate(dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("overlaps"),
            "Expected overlapping path error, got: {}",
            err
        );
    }

    #[test]
    fn validate_accepts_valid_workspace() {
        let dir = setup_workspace(&["api", "worker", "shared"]);

        let toml_str = r#"
            [workspace]
            name = "test"

            [[workspace.members]]
            path = "api"

            [[workspace.members]]
            path = "worker"

            [[workspace.members]]
            path = "shared"
        "#;

        let config: WorkspaceConfig = toml::from_str(toml_str).unwrap();
        config.validate(dir.path()).unwrap();
    }

    #[test]
    fn load_rejects_unsupported_version() {
        let dir = TempDir::new().unwrap();
        let manifest_path = dir.path().join("scope-workspace.toml");
        std::fs::write(
            &manifest_path,
            r#"
            [workspace]
            name = "test"
            version = 99

            [[workspace.members]]
            path = "api"
        "#,
        )
        .unwrap();

        let err = WorkspaceConfig::load(&manifest_path).unwrap_err();
        assert!(
            err.to_string().contains("Unsupported workspace version"),
            "Expected version error, got: {}",
            err
        );
    }

    #[test]
    fn generate_toml_produces_valid_manifest() {
        let members = vec![
            ("services/api".to_string(), "api".to_string()),
            ("libs/shared".to_string(), "shared".to_string()),
        ];
        let toml_str = WorkspaceConfig::generate_toml("my-workspace", &members);

        // Parse it back to verify it's valid TOML
        let config: WorkspaceConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.workspace.name, "my-workspace");
        assert_eq!(config.workspace.members.len(), 2);
        assert_eq!(config.workspace.members[0].path, "services/api");
        assert_eq!(config.workspace.members[0].name, Some("api".to_string()));
    }
}
