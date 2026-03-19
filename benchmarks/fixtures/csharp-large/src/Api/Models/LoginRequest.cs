namespace CSharpLargeApi.Api.Models;

/// <summary>
/// Request model for user authentication.
/// Binds to the JSON body of POST /api/auth/login.
/// </summary>
public class LoginRequest
{
    /// <summary>Gets or sets the email address.</summary>
    public string Email { get; set; } = string.Empty;

    /// <summary>Gets or sets the password.</summary>
    public string Password { get; set; } = string.Empty;
}
