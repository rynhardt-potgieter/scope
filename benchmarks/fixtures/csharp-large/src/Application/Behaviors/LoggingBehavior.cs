using System.Diagnostics;

namespace CSharpLargeApi.Application.Behaviors;

/// <summary>
/// Pipeline behavior that logs command execution with timing information.
/// Captures the command type, execution duration, and any exceptions
/// that occur during processing.
/// </summary>
/// <typeparam name="TCommand">The command type being logged.</typeparam>
/// <typeparam name="TResult">The result type returned by the handler.</typeparam>
public class LoggingBehavior<TCommand, TResult>
{
    private readonly Action<string> _logger;

    /// <summary>
    /// Initializes the behavior with a logging delegate.
    /// </summary>
    public LoggingBehavior(Action<string> logger)
    {
        _logger = logger ?? throw new ArgumentNullException(nameof(logger));
    }

    /// <summary>
    /// Wraps the handler execution with start/end/error logging.
    /// </summary>
    public async Task<TResult> Handle(TCommand command, Func<Task<TResult>> next)
    {
        var commandName = typeof(TCommand).Name;
        var stopwatch = Stopwatch.StartNew();

        _logger($"[START] Handling {commandName}");

        try
        {
            var result = await next();
            stopwatch.Stop();

            _logger($"[END] {commandName} completed in {stopwatch.ElapsedMilliseconds}ms");

            return result;
        }
        catch (Exception ex)
        {
            stopwatch.Stop();
            _logger($"[ERROR] {commandName} failed after {stopwatch.ElapsedMilliseconds}ms: {ex.Message}");
            throw;
        }
    }
}
