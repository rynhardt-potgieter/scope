namespace CSharpLargeApi.Api.Middleware;

/// <summary>
/// Middleware that records audit trail entries for API requests.
/// Captures request metadata (method, path, user, timestamp, duration)
/// for compliance and debugging purposes.
/// </summary>
public class AuditMiddleware
{
    private readonly Func<Task> _next;
    private readonly Action<AuditEntry> _auditWriter;

    /// <summary>
    /// Initializes the middleware with the next delegate and audit writer.
    /// </summary>
    public AuditMiddleware(Func<Task> next, Action<AuditEntry> auditWriter)
    {
        _next = next ?? throw new ArgumentNullException(nameof(next));
        _auditWriter = auditWriter ?? throw new ArgumentNullException(nameof(auditWriter));
    }

    /// <summary>
    /// Invokes the middleware, recording the request as an audit entry.
    /// </summary>
    public async Task InvokeAsync(string method, string path, string? userId)
    {
        var startTime = DateTime.UtcNow;
        var entry = new AuditEntry
        {
            Id = Guid.NewGuid(),
            Method = method,
            Path = path,
            UserId = userId,
            Timestamp = startTime,
            Succeeded = true
        };

        try
        {
            await _next();
        }
        catch (Exception ex)
        {
            entry.Succeeded = false;
            entry.ErrorMessage = ex.Message;
            throw;
        }
        finally
        {
            entry.DurationMs = (long)(DateTime.UtcNow - startTime).TotalMilliseconds;
            _auditWriter(entry);
        }
    }
}

/// <summary>
/// Represents an audit trail entry for an API request.
/// </summary>
public class AuditEntry
{
    /// <summary>
    /// Gets or sets the unique audit entry identifier.
    /// </summary>
    public Guid Id { get; set; }

    /// <summary>
    /// Gets or sets the HTTP method (GET, POST, PUT, DELETE).
    /// </summary>
    public string Method { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the request path.
    /// </summary>
    public string Path { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the authenticated user ID, if available.
    /// </summary>
    public string? UserId { get; set; }

    /// <summary>
    /// Gets or sets the UTC timestamp of the request.
    /// </summary>
    public DateTime Timestamp { get; set; }

    /// <summary>
    /// Gets or sets the request duration in milliseconds.
    /// </summary>
    public long DurationMs { get; set; }

    /// <summary>
    /// Gets or sets whether the request succeeded.
    /// </summary>
    public bool Succeeded { get; set; }

    /// <summary>
    /// Gets or sets the error message if the request failed.
    /// </summary>
    public string? ErrorMessage { get; set; }
}
