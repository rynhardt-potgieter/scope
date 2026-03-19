using System.Diagnostics;

namespace CSharpLargeApi.Application.Behaviors;

/// <summary>
/// Pipeline behavior that monitors handler execution time and logs warnings
/// when handlers exceed a configurable threshold. Helps identify
/// performance regressions in command and query handlers.
/// </summary>
/// <typeparam name="TCommand">The command type being monitored.</typeparam>
/// <typeparam name="TResult">The result type returned by the handler.</typeparam>
public class PerformanceBehavior<TCommand, TResult>
{
    private readonly Action<string> _warningLogger;
    private readonly int _thresholdMilliseconds;

    /// <summary>
    /// Initializes the behavior with a warning logger and performance threshold.
    /// </summary>
    /// <param name="warningLogger">Delegate to log performance warnings.</param>
    /// <param name="thresholdMilliseconds">Execution time threshold in milliseconds. Default: 500ms.</param>
    public PerformanceBehavior(Action<string> warningLogger, int thresholdMilliseconds = 500)
    {
        _warningLogger = warningLogger ?? throw new ArgumentNullException(nameof(warningLogger));
        _thresholdMilliseconds = thresholdMilliseconds;
    }

    /// <summary>
    /// Measures handler execution time and logs a warning if it exceeds the threshold.
    /// </summary>
    public async Task<TResult> Handle(TCommand command, Func<Task<TResult>> next)
    {
        var stopwatch = Stopwatch.StartNew();

        var result = await next();

        stopwatch.Stop();

        if (stopwatch.ElapsedMilliseconds > _thresholdMilliseconds)
        {
            var commandName = typeof(TCommand).Name;
            _warningLogger(
                $"[PERF WARNING] {commandName} took {stopwatch.ElapsedMilliseconds}ms " +
                $"(threshold: {_thresholdMilliseconds}ms). Consider optimizing.");
        }

        return result;
    }
}
