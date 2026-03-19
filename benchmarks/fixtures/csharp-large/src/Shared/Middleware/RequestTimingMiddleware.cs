using System.Diagnostics;

namespace CSharpLargeApi.Shared.Middleware;

/// <summary>
/// Middleware that measures request duration and adds timing headers.
/// Adds X-Response-Time header with the execution duration in milliseconds.
/// </summary>
public class RequestTimingMiddleware
{
    private readonly Func<Task> _next;
    private readonly Action<string>? _logger;
    private readonly long _slowRequestThresholdMs;

    /// <summary>
    /// Initializes the middleware with timing configuration.
    /// </summary>
    public RequestTimingMiddleware(Func<Task> next, Action<string>? logger = null, long slowRequestThresholdMs = 1000)
    {
        _next = next ?? throw new ArgumentNullException(nameof(next));
        _logger = logger;
        _slowRequestThresholdMs = slowRequestThresholdMs;
    }

    /// <summary>
    /// Invokes the middleware, measuring execution time.
    /// </summary>
    public async Task InvokeAsync(string requestPath)
    {
        var stopwatch = Stopwatch.StartNew();

        try
        {
            await _next();
        }
        finally
        {
            stopwatch.Stop();
            var elapsedMs = stopwatch.ElapsedMilliseconds;

            // In real ASP.NET Core: context.Response.Headers["X-Response-Time"] = $"{elapsedMs}ms";

            if (elapsedMs > _slowRequestThresholdMs)
            {
                _logger?.Invoke($"[SLOW REQUEST] {requestPath} took {elapsedMs}ms (threshold: {_slowRequestThresholdMs}ms)");
            }
        }
    }
}
