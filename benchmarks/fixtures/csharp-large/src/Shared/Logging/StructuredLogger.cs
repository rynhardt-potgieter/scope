using System.Collections.Concurrent;

namespace CSharpLargeApi.Shared.Logging;

/// <summary>
/// A structured logger that captures log entries with contextual properties.
/// Provides a consistent logging interface across the application.
/// </summary>
public class StructuredLogger
{
    private readonly string _category;
    private readonly ConcurrentQueue<LogEntry> _entries = new();
    private readonly Action<string>? _outputWriter;

    /// <summary>
    /// Initializes a new structured logger for the specified category.
    /// </summary>
    public StructuredLogger(string category, Action<string>? outputWriter = null)
    {
        _category = category;
        _outputWriter = outputWriter;
    }

    /// <summary>
    /// Logs an informational message.
    /// </summary>
    public void Information(string message, Dictionary<string, object>? properties = null)
    {
        Log(LogLevel.Information, message, properties);
    }

    /// <summary>
    /// Logs a warning message.
    /// </summary>
    public void Warning(string message, Dictionary<string, object>? properties = null)
    {
        Log(LogLevel.Warning, message, properties);
    }

    /// <summary>
    /// Logs an error message with an optional exception.
    /// </summary>
    public void Error(string message, Exception? exception = null, Dictionary<string, object>? properties = null)
    {
        var props = properties ?? new Dictionary<string, object>();
        if (exception is not null)
        {
            props["ExceptionType"] = exception.GetType().Name;
            props["ExceptionMessage"] = exception.Message;
        }
        Log(LogLevel.Error, message, props);
    }

    /// <summary>
    /// Logs a debug message.
    /// </summary>
    public void Debug(string message, Dictionary<string, object>? properties = null)
    {
        Log(LogLevel.Debug, message, properties);
    }

    /// <summary>
    /// Returns all logged entries for inspection.
    /// </summary>
    public IReadOnlyList<LogEntry> GetEntries()
    {
        return _entries.ToList().AsReadOnly();
    }

    private void Log(LogLevel level, string message, Dictionary<string, object>? properties)
    {
        var entry = new LogEntry
        {
            Timestamp = DateTime.UtcNow,
            Level = level,
            Category = _category,
            Message = message,
            Properties = properties ?? new Dictionary<string, object>()
        };

        _entries.Enqueue(entry);
        _outputWriter?.Invoke($"[{entry.Timestamp:HH:mm:ss}] [{entry.Level}] [{entry.Category}] {entry.Message}");
    }
}

/// <summary>
/// Represents a structured log entry.
/// </summary>
public class LogEntry
{
    /// <summary>Gets or sets the timestamp.</summary>
    public DateTime Timestamp { get; set; }

    /// <summary>Gets or sets the log level.</summary>
    public LogLevel Level { get; set; }

    /// <summary>Gets or sets the logger category.</summary>
    public string Category { get; set; } = string.Empty;

    /// <summary>Gets or sets the log message.</summary>
    public string Message { get; set; } = string.Empty;

    /// <summary>Gets or sets the structured properties.</summary>
    public Dictionary<string, object> Properties { get; set; } = new();
}

/// <summary>
/// Log level enumeration for structured logging.
/// </summary>
public enum LogLevel
{
    /// <summary>Detailed debug information.</summary>
    Debug = 0,

    /// <summary>General informational messages.</summary>
    Information = 1,

    /// <summary>Warning messages for potentially harmful situations.</summary>
    Warning = 2,

    /// <summary>Error messages for failures.</summary>
    Error = 3,

    /// <summary>Critical errors that require immediate attention.</summary>
    Critical = 4
}
