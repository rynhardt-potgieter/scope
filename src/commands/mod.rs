/// CLI subcommand implementations for Scope.
///
/// Each module corresponds to one `scope` subcommand.
pub mod deps;
pub mod find;
pub mod impact;
pub mod index;
pub mod init;
pub mod rdeps;
pub mod refs;
pub mod similar;
pub mod sketch;
pub mod source;
pub mod status;

/// Check if an input string looks like a file path rather than a symbol name.
pub fn looks_like_file_path(input: &str) -> bool {
    input.contains('/')
        || input.contains('\\')
        || input.ends_with(".ts")
        || input.ends_with(".tsx")
        || input.ends_with(".js")
        || input.ends_with(".jsx")
        || input.ends_with(".cs")
        || input.ends_with(".rs")
        || input.ends_with(".py")
        || input.ends_with(".go")
        || input.ends_with(".java")
}
