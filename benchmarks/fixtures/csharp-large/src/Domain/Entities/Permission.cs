using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a granular permission that can be assigned to roles.
/// Permissions follow the format "resource:action" (e.g. "payments:create").
/// </summary>
public class Permission : IEntity
{
    /// <summary>
    /// Gets the unique identifier for this permission.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this permission was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this permission was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the permission key in "resource:action" format.
    /// </summary>
    public string Key { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the human-readable description of this permission.
    /// </summary>
    public string Description { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the resource this permission applies to.
    /// </summary>
    public string Resource { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the action this permission grants.
    /// </summary>
    public string Action { get; private set; } = string.Empty;

    /// <summary>
    /// Creates a new permission with the given resource and action.
    /// </summary>
    public static Permission Create(string resource, string action, string description)
    {
        return new Permission
        {
            Id = Guid.NewGuid(),
            Key = $"{resource}:{action}",
            Resource = resource,
            Action = action,
            Description = description,
            CreatedAt = DateTime.UtcNow
        };
    }
}
