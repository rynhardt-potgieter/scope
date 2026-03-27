/// Project configuration stored in `.scope/config.toml`.
///
/// This is the user-facing configuration that controls indexing behaviour,
/// language selection, and output defaults.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Top-level project configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project-level settings.
    pub project: ProjectSection,
    /// Index behaviour settings.
    #[serde(default)]
    pub index: IndexSection,
    /// Embedding provider settings.
    #[serde(default)]
    pub embeddings: EmbeddingsSection,
    /// Output formatting defaults.
    #[serde(default)]
    pub output: OutputSection,
}

/// The `[project]` section of config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSection {
    /// Human-readable project name.
    pub name: String,
    /// Languages to index.
    pub languages: Vec<String>,
}

/// The `[index]` section of config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSection {
    /// Glob patterns to ignore during indexing.
    #[serde(default = "default_ignore_patterns")]
    pub ignore: Vec<String>,
    /// Whether to include test files in refs/impact output.
    #[serde(default = "default_true")]
    pub include_tests: bool,
    /// Path components that identify vendor/dependency code.
    ///
    /// Results from vendor paths are de-ranked (shown after first-party results)
    /// in `scope find` and `scope refs` output. Vendor code is still indexed,
    /// just sorted lower in results.
    #[serde(default = "default_vendor_patterns")]
    pub vendor_patterns: Vec<String>,
}

/// The `[embeddings]` section of config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsSection {
    /// Embedding provider: "local", "voyage", or "openai".
    #[serde(default = "default_provider")]
    pub provider: String,
    /// Model name for the embedding provider.
    #[serde(default = "default_model")]
    pub model: String,
}

/// The `[output]` section of config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSection {
    /// Maximum number of refs to display before truncating.
    #[serde(default = "default_max_refs")]
    pub max_refs: usize,
    /// Maximum depth for impact analysis traversal.
    #[serde(default = "default_max_impact_depth")]
    pub max_impact_depth: usize,
}

impl ProjectConfig {
    /// Load configuration from a `.scope/config.toml` file.
    pub fn load(scope_dir: &Path) -> Result<Self> {
        let config_path = scope_dir.join("config.toml");
        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;
        let config: ProjectConfig =
            toml::from_str(&content).with_context(|| "Failed to parse config.toml")?;
        Ok(config)
    }

    /// Write configuration to a `.scope/config.toml` file.
    pub fn save(&self, scope_dir: &Path) -> Result<()> {
        let config_path = scope_dir.join("config.toml");
        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;
        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write {}", config_path.display()))?;
        Ok(())
    }

    /// Create a default config for a project with detected languages.
    pub fn default_for(name: &str, languages: Vec<String>) -> Self {
        Self {
            project: ProjectSection {
                name: name.to_string(),
                languages,
            },
            index: IndexSection::default(),
            embeddings: EmbeddingsSection::default(),
            output: OutputSection::default(),
        }
    }
}

impl Default for IndexSection {
    fn default() -> Self {
        Self {
            ignore: default_ignore_patterns(),
            include_tests: true,
            vendor_patterns: default_vendor_patterns(),
        }
    }
}

impl Default for EmbeddingsSection {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
        }
    }
}

impl Default for OutputSection {
    fn default() -> Self {
        Self {
            max_refs: default_max_refs(),
            max_impact_depth: default_max_impact_depth(),
        }
    }
}

fn default_ignore_patterns() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".git".to_string(),
    ]
}

fn default_true() -> bool {
    true
}

fn default_provider() -> String {
    "local".to_string()
}

fn default_model() -> String {
    "nomic-embed-code".to_string()
}

fn default_max_refs() -> usize {
    20
}

fn default_max_impact_depth() -> usize {
    3
}

fn default_vendor_patterns() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        "vendor".to_string(),
        "target".to_string(),
        ".cargo".to_string(),
        "venv".to_string(),
        "site-packages".to_string(),
        ".m2".to_string(),
        "third_party".to_string(),
    ]
}

/// Return vendor patterns appropriate for the given set of detected languages.
///
/// Each language contributes its known vendor/build directories. Results
/// are deduplicated so overlapping languages don't produce duplicates.
pub fn vendor_patterns_for_languages(languages: &[String]) -> Vec<String> {
    let mut patterns = Vec::new();

    for lang in languages {
        match lang.as_str() {
            "typescript" => {
                patterns.extend_from_slice(&[
                    "node_modules".to_string(),
                    "dist".to_string(),
                    "build".to_string(),
                ]);
            }
            "rust" => {
                patterns.extend_from_slice(&["target".to_string(), ".cargo".to_string()]);
            }
            "python" => {
                patterns.extend_from_slice(&[
                    "venv".to_string(),
                    ".venv".to_string(),
                    "site-packages".to_string(),
                    "__pycache__".to_string(),
                ]);
            }
            "go" => {
                patterns.push("vendor".to_string());
            }
            "java" => {
                patterns.extend_from_slice(&[
                    ".m2".to_string(),
                    "build".to_string(),
                    "out".to_string(),
                ]);
            }
            "csharp" => {
                patterns.extend_from_slice(&[
                    "bin".to_string(),
                    "obj".to_string(),
                    "packages".to_string(),
                ]);
            }
            _ => {}
        }
    }

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    patterns.retain(|p| seen.insert(p.clone()));
    patterns
}

/// Check whether a file path passes through a vendor directory.
///
/// Returns `true` if any path component exactly matches one of the
/// vendor patterns. Uses path component matching, not substring matching,
/// so `"vendor"` won't match `"my-vendor-lib"`.
pub fn is_vendor_path(file_path: &str, vendor_patterns: &[String]) -> bool {
    let path = std::path::Path::new(file_path);
    path.components().any(|c| {
        if let std::path::Component::Normal(s) = c {
            vendor_patterns
                .iter()
                .any(|p| s.to_str() == Some(p.as_str()))
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_vendor_path_node_modules() {
        let patterns = default_vendor_patterns();
        assert!(is_vendor_path("node_modules/stripe/payment.js", &patterns));
    }

    #[test]
    fn test_is_vendor_path_first_party() {
        let patterns = default_vendor_patterns();
        assert!(!is_vendor_path("src/payments/service.ts", &patterns));
    }

    #[test]
    fn test_is_vendor_path_go_vendor() {
        let patterns = default_vendor_patterns();
        assert!(is_vendor_path(
            "vendor/github.com/pkg/errors/errors.go",
            &patterns
        ));
    }

    #[test]
    fn test_is_vendor_path_no_substring_match() {
        let patterns = default_vendor_patterns();
        assert!(!is_vendor_path("src/my-vendor-lib/util.rs", &patterns));
    }

    #[test]
    fn test_is_vendor_path_nested_vendor() {
        let patterns = default_vendor_patterns();
        assert!(is_vendor_path(
            "packages/api/node_modules/lodash/index.js",
            &patterns
        ));
    }

    #[test]
    fn test_vendor_patterns_for_languages_dedup() {
        let langs = vec!["typescript".to_string(), "java".to_string()];
        let patterns = vendor_patterns_for_languages(&langs);
        let build_count = patterns.iter().filter(|p| p.as_str() == "build").count();
        assert_eq!(build_count, 1);
    }

    #[test]
    fn test_vendor_patterns_for_languages_unknown() {
        let langs = vec!["brainfuck".to_string()];
        let patterns = vendor_patterns_for_languages(&langs);
        assert!(patterns.is_empty());
    }
}
