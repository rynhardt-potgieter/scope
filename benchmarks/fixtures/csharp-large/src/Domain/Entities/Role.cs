using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a named role that groups permissions together.
/// Users are assigned one role which grants them a set of permissions.
/// </summary>
public class Role : IEntity
{
    private readonly List<Permission> _permissions = new();

    /// <summary>
    /// Gets the unique identifier for this role.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this role was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this role was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the name of this role (e.g. "Admin", "Editor", "Viewer").
    /// </summary>
    public string Name { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the description of what this role is for.
    /// </summary>
    public string Description { get; private set; } = string.Empty;

    /// <summary>
    /// Gets whether this is a system-defined role that cannot be deleted.
    /// </summary>
    public bool IsSystemRole { get; private set; }

    /// <summary>
    /// Gets the permissions assigned to this role.
    /// </summary>
    public IReadOnlyList<Permission> Permissions => _permissions.AsReadOnly();

    /// <summary>
    /// Creates a new role with the given name.
    /// </summary>
    public static Role Create(string name, string description, bool isSystemRole = false)
    {
        return new Role
        {
            Id = Guid.NewGuid(),
            Name = name,
            Description = description,
            IsSystemRole = isSystemRole,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Adds a permission to this role.
    /// </summary>
    public void AddPermission(Permission permission)
    {
        if (!_permissions.Any(p => p.Key == permission.Key))
        {
            _permissions.Add(permission);
            UpdatedAt = DateTime.UtcNow;
        }
    }

    /// <summary>
    /// Removes a permission from this role.
    /// </summary>
    public void RemovePermission(string permissionKey)
    {
        var permission = _permissions.FirstOrDefault(p => p.Key == permissionKey);
        if (permission != null)
        {
            _permissions.Remove(permission);
            UpdatedAt = DateTime.UtcNow;
        }
    }

    /// <summary>
    /// Checks whether this role has the specified permission.
    /// </summary>
    public bool HasPermission(string permissionKey)
    {
        return _permissions.Any(p => p.Key == permissionKey);
    }
}
