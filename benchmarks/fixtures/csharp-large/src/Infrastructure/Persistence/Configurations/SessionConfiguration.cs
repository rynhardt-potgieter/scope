using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Session entity.
/// </summary>
public class SessionConfiguration
{
    /// <summary>
    /// Configures the Session entity mapping.
    /// </summary>
    public void Configure()
    {
        var tableName = "Sessions";
        var primaryKey = nameof(Session.Id);

        var indexes = new[]
        {
            "IX_Sessions_UserId",
            "IX_Sessions_RefreshToken",
            "IX_Sessions_ExpiresAt"
        };

        var maxLengths = new Dictionary<string, int>
        {
            { nameof(Session.RefreshToken), 512 },
            { nameof(Session.IpAddress), 45 },
            { nameof(Session.UserAgent), 500 },
        };

        _ = tableName;
        _ = primaryKey;
        _ = indexes;
        _ = maxLengths;
    }
}
