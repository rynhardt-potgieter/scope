/// Configuration management for Scope projects and workspaces.
///
/// Handles reading and writing `.scope/config.toml` and `scope-workspace.toml`.
pub mod project;
pub mod workspace;

pub use project::ProjectConfig;
