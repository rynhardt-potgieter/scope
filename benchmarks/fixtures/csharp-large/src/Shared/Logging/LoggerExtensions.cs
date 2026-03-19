namespace CSharpLargeApi.Shared.Logging;

/// <summary>
/// Extension methods for structured logging with consistent formatting.
/// Provides convenience methods for common log patterns.
/// </summary>
public static class LoggerExtensions
{
    /// <summary>
    /// Logs the start of an operation with context information.
    /// </summary>
    public static void LogOperationStart(this Action<string> logger, string operationName, string? correlationId = null)
    {
        var prefix = correlationId is not null ? $"[{correlationId}] " : "";
        logger($"{prefix}[START] {operationName}");
    }

    /// <summary>
    /// Logs the successful completion of an operation with duration.
    /// </summary>
    public static void LogOperationEnd(this Action<string> logger, string operationName, long durationMs, string? correlationId = null)
    {
        var prefix = correlationId is not null ? $"[{correlationId}] " : "";
        logger($"{prefix}[END] {operationName} completed in {durationMs}ms");
    }

    /// <summary>
    /// Logs an operation failure with error details.
    /// </summary>
    public static void LogOperationError(this Action<string> logger, string operationName, Exception ex, string? correlationId = null)
    {
        var prefix = correlationId is not null ? $"[{correlationId}] " : "";
        logger($"{prefix}[ERROR] {operationName} failed: {ex.GetType().Name}: {ex.Message}");
    }

    /// <summary>
    /// Logs a business event for audit trail purposes.
    /// </summary>
    public static void LogBusinessEvent(this Action<string> logger, string eventName, Guid entityId, string? userId = null)
    {
        var userPart = userId is not null ? $" by user {userId}" : "";
        logger($"[EVENT] {eventName} for entity {entityId}{userPart}");
    }

    /// <summary>
    /// Logs a performance warning when an operation exceeds a threshold.
    /// </summary>
    public static void LogPerformanceWarning(this Action<string> logger, string operationName, long durationMs, long thresholdMs)
    {
        logger($"[PERF] {operationName} took {durationMs}ms (threshold: {thresholdMs}ms)");
    }
}
