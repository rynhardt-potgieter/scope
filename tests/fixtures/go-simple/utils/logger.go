package utils

import "fmt"

// Logger handles structured logging.
type Logger struct {
	prefix string
}

// NewLogger creates a new Logger with the given prefix.
func NewLogger(prefix string) Logger {
	return Logger{prefix: prefix}
}

// Info logs an informational message.
func (l *Logger) Info(msg string) {
	fmt.Printf("[%s] INFO: %s\n", l.prefix, msg)
}

// Error logs an error message.
func (l *Logger) Error(msg string) {
	fmt.Printf("[%s] ERROR: %s\n", l.prefix, msg)
}

// Debug logs a debug message (unexported).
func (l *Logger) Debug(msg string) {
	fmt.Printf("[%s] DEBUG: %s\n", l.prefix, msg)
}

// formatMessage is an unexported helper function.
func formatMessage(prefix string, level string, msg string) string {
	return fmt.Sprintf("[%s] %s: %s", prefix, level, msg)
}
