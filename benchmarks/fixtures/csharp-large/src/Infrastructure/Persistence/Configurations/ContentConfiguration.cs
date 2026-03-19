using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Content entity.
/// </summary>
public class ContentConfiguration
{
    /// <summary>
    /// Configures the Content entity mapping.
    /// </summary>
    public void Configure()
    {
        var tableName = "Contents";
        var primaryKey = nameof(Content.Id);

        var requiredProperties = new[]
        {
            nameof(Content.Title),
            nameof(Content.Slug),
            nameof(Content.Body),
            nameof(Content.AuthorId),
        };

        var maxLengths = new Dictionary<string, int>
        {
            { nameof(Content.Title), 200 },
            { nameof(Content.Slug), 200 },
        };

        var indexes = new[]
        {
            "IX_Contents_Slug",
            "IX_Contents_AuthorId",
            "IX_Contents_Status",
            "IX_Contents_PublishedAt"
        };

        // Many-to-many relationship with Tags via ContentTags join table
        var joinTable = "ContentTags";

        _ = tableName;
        _ = primaryKey;
        _ = requiredProperties;
        _ = maxLengths;
        _ = indexes;
        _ = joinTable;
    }
}
