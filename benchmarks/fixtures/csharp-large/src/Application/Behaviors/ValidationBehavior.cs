namespace CSharpLargeApi.Application.Behaviors;

/// <summary>
/// Pipeline behavior that validates commands before they reach their handlers.
/// If validation fails, a ValidationException is thrown with all errors
/// before any business logic executes.
/// </summary>
/// <typeparam name="TCommand">The command type being validated.</typeparam>
/// <typeparam name="TResult">The result type returned by the handler.</typeparam>
public class ValidationBehavior<TCommand, TResult>
{
    private readonly Func<TCommand, IReadOnlyList<string>>? _validator;

    /// <summary>
    /// Initializes the behavior with an optional validator function.
    /// </summary>
    public ValidationBehavior(Func<TCommand, IReadOnlyList<string>>? validator = null)
    {
        _validator = validator;
    }

    /// <summary>
    /// Executes validation before passing the command to the next handler in the pipeline.
    /// Throws an InvalidOperationException if validation errors are found.
    /// </summary>
    public async Task<TResult> Handle(TCommand command, Func<Task<TResult>> next)
    {
        if (_validator is not null)
        {
            var errors = _validator(command);
            if (errors.Count > 0)
            {
                var errorMessage = string.Join("; ", errors);
                throw new InvalidOperationException($"Validation failed: {errorMessage}");
            }
        }

        return await next();
    }
}
