using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Subscription entity.
/// </summary>
public class SubscriptionConfiguration
{
    /// <summary>
    /// Configures the Subscription entity mapping.
    /// </summary>
    public void Configure()
    {
        var tableName = "Subscriptions";
        var primaryKey = nameof(Subscription.Id);

        var indexes = new[]
        {
            "IX_Subscriptions_UserId",
            "IX_Subscriptions_Status",
            "IX_Subscriptions_NextRenewalDate"
        };

        var requiredProperties = new[]
        {
            nameof(Subscription.PlanName),
            nameof(Subscription.UserId),
        };

        _ = tableName;
        _ = primaryKey;
        _ = indexes;
        _ = requiredProperties;
    }
}
