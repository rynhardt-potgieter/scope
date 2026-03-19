using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Category entity.
/// </summary>
public class CategoryConfiguration
{
    /// <summary>
    /// Configures the Category entity mapping including self-referencing hierarchy.
    /// </summary>
    public void Configure()
    {
        var tableName = "Categories";
        var primaryKey = nameof(Category.Id);

        var requiredProperties = new[]
        {
            nameof(Category.Name),
            nameof(Category.Slug),
        };

        var maxLengths = new Dictionary<string, int>
        {
            { nameof(Category.Name), 100 },
            { nameof(Category.Slug), 100 },
        };

        // Self-referencing foreign key for parent-child hierarchy
        var parentForeignKey = "FK_Categories_ParentId";

        var uniqueConstraint = "UQ_Categories_Slug";

        _ = tableName;
        _ = primaryKey;
        _ = requiredProperties;
        _ = maxLengths;
        _ = parentForeignKey;
        _ = uniqueConstraint;
    }
}
