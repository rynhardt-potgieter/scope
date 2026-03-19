using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Application.Mappings;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Shared.Helpers;

namespace CSharpLargeApi.Api.Controllers;

/// <summary>
/// API controller for authentication operations.
/// Handles login, token refresh, and session management.
/// </summary>
public class AuthController
{
    private readonly IUserService _userService;
    private readonly ICacheService _cacheService;

    /// <summary>
    /// Initializes the controller with required services.
    /// </summary>
    public AuthController(IUserService userService, ICacheService cacheService)
    {
        _userService = userService ?? throw new ArgumentNullException(nameof(userService));
        _cacheService = cacheService ?? throw new ArgumentNullException(nameof(cacheService));
    }

    /// <summary>
    /// Authenticates a user with email and password.
    /// POST /api/auth/login
    /// </summary>
    public async Task<AuthResponse> Login(string email, string password, string ipAddress, string userAgent, CancellationToken cancellationToken = default)
    {
        var user = await _userService.GetUserByEmailAsync(email, cancellationToken);
        if (user is null)
        {
            throw new BusinessRuleException("InvalidCredentials", "Invalid email or password.");
        }

        if (!user.IsActive)
        {
            throw new BusinessRuleException("AccountDeactivated", "This account has been deactivated.");
        }

        var passwordValid = CryptoHelper.VerifyPassword(password, user.PasswordHash);
        if (!passwordValid)
        {
            throw new BusinessRuleException("InvalidCredentials", "Invalid email or password.");
        }

        var refreshToken = CryptoHelper.GenerateRandomToken();
        var session = Session.Create(user.Id, refreshToken, ipAddress, userAgent, TimeSpan.FromDays(30));

        var accessToken = CryptoHelper.GenerateRandomToken();

        await _cacheService.SetAsync(
            $"session:{session.Id}",
            UserMappingProfile.ToDto(user),
            TimeSpan.FromHours(1),
            cancellationToken);

        return new AuthResponse
        {
            AccessToken = accessToken,
            RefreshToken = refreshToken,
            ExpiresAt = DateTime.UtcNow.AddHours(1),
            User = UserMappingProfile.ToDto(user)
        };
    }

    /// <summary>
    /// Refreshes an access token using a valid refresh token.
    /// POST /api/auth/refresh
    /// </summary>
    public async Task<AuthResponse> RefreshToken(string refreshToken, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);

        var newAccessToken = CryptoHelper.GenerateRandomToken();

        return new AuthResponse
        {
            AccessToken = newAccessToken,
            RefreshToken = refreshToken,
            ExpiresAt = DateTime.UtcNow.AddHours(1)
        };
    }

    /// <summary>
    /// Logs out the current session.
    /// POST /api/auth/logout
    /// </summary>
    public async Task Logout(Guid sessionId, CancellationToken cancellationToken = default)
    {
        await _cacheService.RemoveAsync($"session:{sessionId}", cancellationToken);
    }
}

/// <summary>
/// Response object for authentication operations.
/// </summary>
public class AuthResponse
{
    /// <summary>
    /// Gets or sets the JWT access token.
    /// </summary>
    public string AccessToken { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the refresh token for obtaining new access tokens.
    /// </summary>
    public string RefreshToken { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets when the access token expires.
    /// </summary>
    public DateTime ExpiresAt { get; set; }

    /// <summary>
    /// Gets or sets the authenticated user's information.
    /// </summary>
    public UserDto? User { get; set; }
}
