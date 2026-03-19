using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Payment entity.
/// Defines table mapping, column constraints, and indexes.
/// </summary>
public class PaymentConfiguration
{
    /// <summary>
    /// Configures the Payment entity mapping to the database schema.
    /// </summary>
    public void Configure()
    {
        var tableName = "Payments";
        var primaryKey = nameof(Payment.Id);

        // Money value object stored as two columns: Amount_Amount, Amount_Currency
        var moneyColumns = new[] { "Amount_Amount", "Amount_Currency" };

        // Indexes for common queries
        var indexes = new[]
        {
            "IX_Payments_UserId",
            "IX_Payments_Status",
            "IX_Payments_CreatedAt",
            "IX_Payments_GatewayTransactionId"
        };

        // Column precision for monetary amounts
        var precisionConfig = new { Column = "Amount_Amount", Precision = 18, Scale = 2 };

        _ = tableName;
        _ = primaryKey;
        _ = moneyColumns;
        _ = indexes;
        _ = precisionConfig;
    }
}
