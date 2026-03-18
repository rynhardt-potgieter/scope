namespace CSharpApi.Users;

/// <summary>
/// Represents a user in the system.
/// </summary>
public class User
{
    /// <summary>Unique identifier for the user.</summary>
    public string Id { get; set; } = string.Empty;

    /// <summary>User's email address.</summary>
    public string Email { get; set; } = string.Empty;

    /// <summary>User's display name.</summary>
    public string Name { get; set; } = string.Empty;

    /// <summary>When the user account was created.</summary>
    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;

    /// <summary>Whether the user account is active.</summary>
    public bool IsActive { get; set; } = true;
}
