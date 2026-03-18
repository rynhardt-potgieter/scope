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
