namespace CSharpLargeApi.Api.Models;

/// <summary>
/// Request model for creating a new user account.
/// Binds to the JSON body of POST /api/users.
/// </summary>
public class CreateUserRequest
{
    /// <summary>Gets or sets the display name.</summary>
    public string DisplayName { get; set; } = string.Empty;

    /// <summary>Gets or sets the email address.</summary>
    public string Email { get; set; } = string.Empty;

    /// <summary>Gets or sets the password.</summary>
    public string Password { get; set; } = string.Empty;

    /// <summary>Gets or sets the role name.</summary>
    public string Role { get; set; } = "User";

    /// <summary>Gets or sets the optional phone number.</summary>
    public string? PhoneNumber { get; set; }
}
