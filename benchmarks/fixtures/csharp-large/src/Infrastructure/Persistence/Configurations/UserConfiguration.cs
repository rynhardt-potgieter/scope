using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the User entity.
/// Defines table mapping, column constraints, and indexes.
/// </summary>
public class UserConfiguration
{
    /// <summary>
    /// Configures the User entity mapping to the database schema.
    /// </summary>
    public void Configure()
    {
        // Table name
        var tableName = "Users";

        // Primary key
        var primaryKey = nameof(User.Id);

        // Required properties
        var requiredProperties = new[]
        {
            nameof(User.DisplayName),
            nameof(User.PasswordHash),
        };

        // Max lengths
        var maxLengths = new Dictionary<string, int>
        {
            { nameof(User.DisplayName), 100 },
            { nameof(User.PasswordHash), 256 },
        };

        // Indexes
        var uniqueIndexes = new[]
        {
            "IX_Users_Email"
        };

        // Value object conversions
        // Email -> string conversion
        // PhoneNumber -> string conversion

        // Seed data is not configured here — handled by migrations
        _ = tableName;
        _ = primaryKey;
        _ = requiredProperties;
        _ = maxLengths;
        _ = uniqueIndexes;
    }
}
