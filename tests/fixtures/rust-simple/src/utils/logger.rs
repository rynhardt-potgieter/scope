/// A simple logger for the payment system.
pub struct Logger {
    prefix: String,
}

impl Logger {
    /// Create a new logger with a prefix.
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    /// Log an info message.
    pub fn info(&self, message: &str) {
        println!("[{}] INFO: {}", self.prefix, message);
    }

    /// Log a warning message.
    pub fn warn(&self, message: &str) {
        eprintln!("[{}] WARN: {}", self.prefix, message);
    }

    /// Log an error message.
    pub fn error(&self, message: &str) {
        eprintln!("[{}] ERROR: {}", self.prefix, message);
    }
}

/// Static logger instance name.
pub static DEFAULT_LOGGER_NAME: &str = "scope";
