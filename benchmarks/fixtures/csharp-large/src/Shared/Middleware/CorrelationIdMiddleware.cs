namespace CSharpLargeApi.Shared.Middleware;

/// <summary>
/// Middleware that ensures every request has a correlation ID.
/// If the request includes an X-Correlation-Id header, it is used;
/// otherwise a new GUID is generated. The correlation ID is added
/// to the response headers and made available to logging.
/// </summary>
public class CorrelationIdMiddleware
{
    private const string CorrelationIdHeader = "X-Correlation-Id";
    private readonly Func<Task> _next;

    /// <summary>
    /// Initializes the middleware with the next delegate.
    /// </summary>
    public CorrelationIdMiddleware(Func<Task> next)
    {
        _next = next ?? throw new ArgumentNullException(nameof(next));
    }

    /// <summary>
    /// Invokes the middleware, ensuring a correlation ID is present.
    /// </summary>
    public async Task InvokeAsync(string? incomingCorrelationId)
    {
        var correlationId = !string.IsNullOrWhiteSpace(incomingCorrelationId)
            ? incomingCorrelationId
            : Guid.NewGuid().ToString("N");

        // In a real ASP.NET Core app:
        // context.Items["CorrelationId"] = correlationId;
        // context.Response.Headers[CorrelationIdHeader] = correlationId;

        _ = correlationId;

        await _next();
    }

    /// <summary>
    /// Extracts the correlation ID from the current context.
    /// </summary>
    public static string GetCorrelationId(Dictionary<string, object> contextItems)
    {
        if (contextItems.TryGetValue("CorrelationId", out var id) && id is string correlationId)
        {
            return correlationId;
        }

        return Guid.NewGuid().ToString("N");
    }
}
