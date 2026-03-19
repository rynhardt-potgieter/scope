using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Notification entity.
/// </summary>
public class NotificationConfiguration
{
    /// <summary>
    /// Configures the Notification entity mapping.
    /// </summary>
    public void Configure()
    {
        var tableName = "Notifications";
        var primaryKey = nameof(Notification.Id);

        var indexes = new[]
        {
            "IX_Notifications_RecipientId",
            "IX_Notifications_IsDelivered",
            "IX_Notifications_IsRead",
            "IX_Notifications_CreatedAt"
        };

        var maxLengths = new Dictionary<string, int>
        {
            { nameof(Notification.Subject), 200 },
        };

        _ = tableName;
        _ = primaryKey;
        _ = indexes;
        _ = maxLengths;
    }
}
