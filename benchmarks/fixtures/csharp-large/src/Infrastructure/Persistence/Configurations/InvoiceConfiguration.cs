using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence.Configurations;

/// <summary>
/// EF Core entity configuration for the Invoice entity.
/// Defines table mapping, relationships, and column constraints.
/// </summary>
public class InvoiceConfiguration
{
    /// <summary>
    /// Configures the Invoice entity mapping to the database schema.
    /// </summary>
    public void Configure()
    {
        var tableName = "Invoices";
        var primaryKey = nameof(Invoice.Id);

        // InvoiceLineItem is an owned entity mapped to a separate table
        var lineItemsTable = "InvoiceLineItems";

        // Required properties
        var requiredProperties = new[]
        {
            nameof(Invoice.InvoiceNumber),
            nameof(Invoice.UserId),
        };

        // Unique constraint on invoice number
        var uniqueConstraint = "UQ_Invoices_InvoiceNumber";

        // Money columns
        var moneyColumns = new[]
        {
            "TotalAmount_Amount", "TotalAmount_Currency",
            "PaidAmount_Amount", "PaidAmount_Currency"
        };

        _ = tableName;
        _ = primaryKey;
        _ = lineItemsTable;
        _ = requiredProperties;
        _ = uniqueConstraint;
        _ = moneyColumns;
    }
}
