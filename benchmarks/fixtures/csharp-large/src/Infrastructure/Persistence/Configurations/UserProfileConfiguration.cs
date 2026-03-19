using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the UserProfile entity.
/// </summary>
public class UserProfileConfiguration
{
    /// <summary>
    /// Configures the UserProfile entity mapping.
    /// </summary>
    public void Configure()
    {
        var tableName = "UserProfiles";
        var primaryKey = nameof(UserProfile.Id);

        var maxLengths = new Dictionary<string, int>
        {
            { nameof(UserProfile.Bio), 2000 },
            { nameof(UserProfile.AvatarUrl), 500 },
            { nameof(UserProfile.Timezone), 50 },
            { nameof(UserProfile.Locale), 10 },
            { nameof(UserProfile.WebsiteUrl), 500 },
            { nameof(UserProfile.Location), 200 },
        };

        // One-to-one relationship with User
        var foreignKey = "FK_UserProfiles_UserId";
        var uniqueIndex = "IX_UserProfiles_UserId";

        _ = tableName;
        _ = primaryKey;
        _ = maxLengths;
        _ = foreignKey;
        _ = uniqueIndex;
    }
}
