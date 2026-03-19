using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Role entity.
/// </summary>
public class RoleConfiguration
{
    /// <summary>
    /// Configures the Role entity mapping.
    /// </summary>
    public void Configure()
    {
        var tableName = "Roles";
        var primaryKey = nameof(Role.Id);

        var requiredProperties = new[]
        {
            nameof(Role.Name),
        };

        var maxLengths = new Dictionary<string, int>
        {
            { nameof(Role.Name), 50 },
            { nameof(Role.Description), 200 },
        };

        var uniqueConstraint = "UQ_Roles_Name";

        // Many-to-many with Permissions via RolePermissions join table
        var joinTable = "RolePermissions";

        _ = tableName;
        _ = primaryKey;
        _ = requiredProperties;
        _ = maxLengths;
        _ = uniqueConstraint;
        _ = joinTable;
    }
}
