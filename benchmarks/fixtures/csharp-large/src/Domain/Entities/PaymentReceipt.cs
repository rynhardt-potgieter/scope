using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a receipt generated for a completed payment.
/// Contains summary information for customer-facing documentation.
/// </summary>
public class PaymentReceipt : IAggregateRoot
{
    private readonly List<IDomainEvent> _domainEvents = new();

    /// <summary>
    /// Gets the unique identifier for this receipt.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this receipt was generated.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this receipt was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the ID of the payment this receipt is for.
    /// </summary>
    public Guid PaymentId { get; private set; }

    /// <summary>
    /// Gets the ID of the user who made the payment.
    /// </summary>
    public Guid UserId { get; private set; }

    /// <summary>
    /// Gets the receipt number for display purposes.
    /// </summary>
    public string ReceiptNumber { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the payment amount.
    /// </summary>
    public Money Amount { get; private set; } = null!;

    /// <summary>
    /// Gets the payment method description (e.g. "Visa ending in 4242").
    /// </summary>
    public string PaymentMethodDescription { get; private set; } = string.Empty;

    /// <summary>
    /// Gets whether this receipt has been sent to the customer.
    /// </summary>
    public bool IsSent { get; private set; }

    /// <summary>
    /// Gets the collection of domain events raised by this aggregate.
    /// </summary>
    public IReadOnlyCollection<IDomainEvent> DomainEvents => _domainEvents.AsReadOnly();

    /// <summary>
    /// Creates a new payment receipt for the specified payment.
    /// </summary>
    public static PaymentReceipt Create(Guid paymentId, Guid userId, Money amount, string paymentMethodDescription)
    {
        return new PaymentReceipt
        {
            Id = Guid.NewGuid(),
            PaymentId = paymentId,
            UserId = userId,
            ReceiptNumber = $"RCP-{DateTime.UtcNow:yyyyMMdd}-{Guid.NewGuid().ToString("N")[..8].ToUpper()}",
            Amount = amount,
            PaymentMethodDescription = paymentMethodDescription,
            IsSent = false,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Marks this receipt as sent to the customer.
    /// </summary>
    public void MarkAsSent()
    {
        IsSent = true;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Clears all pending domain events from this aggregate.
    /// </summary>
    public void ClearDomainEvents()
    {
        _domainEvents.Clear();
    }
}
