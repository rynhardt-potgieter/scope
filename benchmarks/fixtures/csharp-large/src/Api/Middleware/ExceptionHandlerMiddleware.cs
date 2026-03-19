using CSharpLargeApi.Domain.Exceptions;

namespace CSharpLargeApi.Api.Middleware;

/// <summary>
/// Middleware that catches unhandled exceptions and converts them to
/// appropriate HTTP responses. Maps domain exceptions to HTTP status codes.
/// </summary>
public class ExceptionHandlerMiddleware
{
    private readonly Func<Task> _next;
    private readonly Action<string> _logger;

    /// <summary>
    /// Initializes the middleware with the next delegate and logger.
    /// </summary>
    public ExceptionHandlerMiddleware(Func<Task> next, Action<string> logger)
    {
        _next = next ?? throw new ArgumentNullException(nameof(next));
        _logger = logger ?? throw new ArgumentNullException(nameof(logger));
    }

    /// <summary>
    /// Invokes the middleware, catching and handling any exceptions.
    /// </summary>
    public async Task InvokeAsync()
    {
        try
        {
            await _next();
        }
        catch (EntityNotFoundException ex)
        {
            _logger($"Entity not found: {ex.EntityType} with ID {ex.EntityId}");
            // Return 404
        }
        catch (BusinessRuleException ex)
        {
            _logger($"Business rule violation: {ex.GetDetailedMessage()}");
            // Return 422
        }
        catch (UnauthorizedAccessException ex)
        {
            _logger($"Unauthorized access: {ex.Message}");
            // Return 403
        }
        catch (InvalidOperationException ex)
        {
            _logger($"Validation error: {ex.Message}");
            // Return 400
        }
        catch (Exception ex)
        {
            _logger($"Unhandled exception: {ex.GetType().Name}: {ex.Message}");
            // Return 500
        }
    }

    /// <summary>
    /// Maps an exception to an HTTP status code.
    /// </summary>
    public static int GetStatusCode(Exception exception)
    {
        return exception switch
        {
            EntityNotFoundException => 404,
            BusinessRuleException => 422,
            UnauthorizedAccessException => 403,
            InvalidOperationException => 400,
            ArgumentException => 400,
            _ => 500
        };
    }
}
