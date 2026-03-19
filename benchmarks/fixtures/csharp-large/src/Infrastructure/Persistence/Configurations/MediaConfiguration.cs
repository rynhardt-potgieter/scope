using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Media entity.
/// </summary>
public class MediaConfiguration
{
    /// <summary>
    /// Configures the Media entity mapping.
    /// </summary>
    public void Configure()
    {
        var tableName = "Media";
        var primaryKey = nameof(Media.Id);

        var requiredProperties = new[]
        {
            nameof(Media.FileName),
            nameof(Media.ContentType),
            nameof(Media.StoragePath),
        };

        var maxLengths = new Dictionary<string, int>
        {
            { nameof(Media.FileName), 256 },
            { nameof(Media.ContentType), 100 },
            { nameof(Media.StoragePath), 1024 },
            { nameof(Media.AltText), 500 },
        };

        var indexes = new[]
        {
            "IX_Media_UploadedById",
            "IX_Media_ContentType"
        };

        _ = tableName;
        _ = primaryKey;
        _ = requiredProperties;
        _ = maxLengths;
        _ = indexes;
    }
}
