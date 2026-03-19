using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Tag entity.
/// </summary>
public class TagConfiguration
{
    /// <summary>
    /// Configures the Tag entity mapping.
    /// </summary>
    public void Configure()
    {
        var tableName = "Tags";
        var primaryKey = nameof(Tag.Id);

        var requiredProperties = new[]
        {
            nameof(Tag.Name),
            nameof(Tag.Slug),
        };

        var maxLengths = new Dictionary<string, int>
        {
            { nameof(Tag.Name), 50 },
            { nameof(Tag.Slug), 50 },
        };

        var uniqueConstraint = "UQ_Tags_Slug";

        _ = tableName;
        _ = primaryKey;
        _ = requiredProperties;
        _ = maxLengths;
        _ = uniqueConstraint;
    }
}
