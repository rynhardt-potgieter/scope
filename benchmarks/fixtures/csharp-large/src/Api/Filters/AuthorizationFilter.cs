namespace CSharpLargeApi.Api.Filters;

/// <summary>
/// Action filter that checks user permissions before allowing access
/// to protected endpoints. Uses a permission-based access control model.
/// </summary>
public class AuthorizationFilter
{
    private readonly HashSet<string> _requiredPermissions;

    /// <summary>
    /// Initializes the filter with the required permissions for the endpoint.
    /// </summary>
    public AuthorizationFilter(params string[] requiredPermissions)
    {
        _requiredPermissions = new HashSet<string>(requiredPermissions);
    }

    /// <summary>
    /// Checks whether the given set of user permissions satisfies all requirements.
    /// </summary>
    public AuthorizationResult Authorize(IEnumerable<string> userPermissions)
    {
        var userPerms = new HashSet<string>(userPermissions);

        var missingPermissions = _requiredPermissions
            .Where(p => !userPerms.Contains(p))
            .ToList();

        if (missingPermissions.Count == 0)
        {
            return AuthorizationResult.Allowed();
        }

        return AuthorizationResult.Denied(
            $"Missing required permissions: {string.Join(", ", missingPermissions)}");
    }

    /// <summary>
    /// Checks whether the user has at least one of the required permissions.
    /// </summary>
    public bool HasAnyPermission(IEnumerable<string> userPermissions)
    {
        var userPerms = new HashSet<string>(userPermissions);
        return _requiredPermissions.Any(p => userPerms.Contains(p));
    }
}

/// <summary>
/// Represents the result of an authorization check.
/// </summary>
public class AuthorizationResult
{
    /// <summary>
    /// Gets whether authorization was granted.
    /// </summary>
    public bool IsAllowed { get; private set; }

    /// <summary>
    /// Gets the denial reason if authorization was not granted.
    /// </summary>
    public string? DenialReason { get; private set; }

    /// <summary>
    /// Creates an allowed authorization result.
    /// </summary>
    public static AuthorizationResult Allowed() => new() { IsAllowed = true };

    /// <summary>
    /// Creates a denied authorization result with a reason.
    /// </summary>
    public static AuthorizationResult Denied(string reason) => new() { IsAllowed = false, DenialReason = reason };
}
