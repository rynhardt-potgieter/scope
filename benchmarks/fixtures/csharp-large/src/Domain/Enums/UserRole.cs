namespace CSharpLargeApi.Domain.Enums;

/// <summary>
/// Defines the broad role categories available to platform users.
/// Fine-grained permissions are managed through the Permission entity.
/// </summary>
public enum UserRole
{
    /// <summary>Standard user with basic access.</summary>
    User = 0,

    /// <summary>Content editor with publishing rights.</summary>
    Editor = 1,

    /// <summary>Moderator with content and user management access.</summary>
    Moderator = 2,

    /// <summary>Full administrative access.</summary>
    Admin = 3,

    /// <summary>System-level super admin (internal use only).</summary>
    SuperAdmin = 4
}
