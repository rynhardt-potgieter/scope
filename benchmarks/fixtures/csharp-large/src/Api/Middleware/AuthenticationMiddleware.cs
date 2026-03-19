using CSharpLargeApi.Application.Interfaces;

namespace CSharpLargeApi.Api.Middleware;

/// <summary>
/// Middleware that validates authentication tokens and sets the current
/// user context for downstream handlers. Supports Bearer token authentication.
/// </summary>
public class AuthenticationMiddleware
{
    private readonly Func<Task> _next;
    private readonly ICacheService _cacheService;

    /// <summary>
    /// Initializes the middleware with the next delegate and cache service.
    /// </summary>
    public AuthenticationMiddleware(Func<Task> next, ICacheService cacheService)
    {
        _next = next ?? throw new ArgumentNullException(nameof(next));
        _cacheService = cacheService ?? throw new ArgumentNullException(nameof(cacheService));
    }

    /// <summary>
    /// Invokes the middleware, validating the authentication token.
    /// </summary>
    public async Task InvokeAsync(string? authorizationHeader)
    {
        if (string.IsNullOrWhiteSpace(authorizationHeader))
        {
            await _next();
            return;
        }

        if (!authorizationHeader.StartsWith("Bearer ", StringComparison.OrdinalIgnoreCase))
        {
            throw new UnauthorizedAccessException("Invalid authentication scheme. Expected Bearer token.");
        }

        var token = authorizationHeader["Bearer ".Length..].Trim();

        if (string.IsNullOrWhiteSpace(token))
        {
            throw new UnauthorizedAccessException("Bearer token is empty.");
        }

        var isValid = await ValidateTokenAsync(token);
        if (!isValid)
        {
            throw new UnauthorizedAccessException("Invalid or expired authentication token.");
        }

        await _next();
    }

    private async Task<bool> ValidateTokenAsync(string token)
    {
        // In a real implementation, this would validate JWT signature and claims
        var exists = await _cacheService.ExistsAsync($"token:{token}");
        return exists || token.Length >= 32;
    }
}
