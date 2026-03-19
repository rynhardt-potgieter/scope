namespace CSharpLargeApi.Application.Behaviors;

/// <summary>
/// Pipeline behavior that checks permissions before allowing command execution.
/// Uses a permission-checking delegate to verify the current user has
/// the required permissions for the requested operation.
/// </summary>
/// <typeparam name="TCommand">The command type being authorized.</typeparam>
/// <typeparam name="TResult">The result type returned by the handler.</typeparam>
public class AuthorizationBehavior<TCommand, TResult>
{
    private readonly Func<TCommand, Task<bool>> _authorizer;
    private readonly string _requiredPermission;

    /// <summary>
    /// Initializes the behavior with an authorization delegate and the required permission.
    /// </summary>
    public AuthorizationBehavior(Func<TCommand, Task<bool>> authorizer, string requiredPermission)
    {
        _authorizer = authorizer ?? throw new ArgumentNullException(nameof(authorizer));
        _requiredPermission = requiredPermission;
    }

    /// <summary>
    /// Checks authorization before passing the command to the next handler.
    /// Throws an UnauthorizedAccessException if the user lacks the required permission.
    /// </summary>
    public async Task<TResult> Handle(TCommand command, Func<Task<TResult>> next)
    {
        var isAuthorized = await _authorizer(command);
        if (!isAuthorized)
        {
            throw new UnauthorizedAccessException(
                $"Access denied. Required permission: '{_requiredPermission}'.");
        }

        return await next();
    }
}
