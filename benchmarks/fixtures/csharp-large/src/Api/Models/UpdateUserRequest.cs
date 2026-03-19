namespace CSharpLargeApi.Api.Models;

/// <summary>
/// Request model for updating user profile information.
/// Binds to the JSON body of PUT /api/users/{id}.
/// </summary>
public class UpdateUserRequest
{
    /// <summary>Gets or sets the updated display name.</summary>
    public string DisplayName { get; set; } = string.Empty;

    /// <summary>Gets or sets the optional updated phone number.</summary>
    public string? PhoneNumber { get; set; }

    /// <summary>Gets or sets the optional updated bio.</summary>
    public string? Bio { get; set; }

    /// <summary>Gets or sets the optional timezone preference.</summary>
    public string? Timezone { get; set; }

    /// <summary>Gets or sets the optional locale preference.</summary>
    public string? Locale { get; set; }
}
