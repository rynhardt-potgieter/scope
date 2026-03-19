using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Permission entity.
/// </summary>
public class PermissionConfiguration
{
    /// <summary>
    /// Configures the Permission entity mapping.
    /// </summary>
    public void Configure()
    {
        var tableName = "Permissions";
        var primaryKey = nameof(Permission.Id);

        var requiredProperties = new[]
        {
            nameof(Permission.Key),
            nameof(Permission.Resource),
            nameof(Permission.Action),
        };

        var maxLengths = new Dictionary<string, int>
        {
            { nameof(Permission.Key), 100 },
            { nameof(Permission.Resource), 50 },
            { nameof(Permission.Action), 50 },
            { nameof(Permission.Description), 200 },
        };

        var uniqueConstraint = "UQ_Permissions_Key";

        _ = tableName;
        _ = primaryKey;
        _ = requiredProperties;
        _ = maxLengths;
        _ = uniqueConstraint;
    }
}
