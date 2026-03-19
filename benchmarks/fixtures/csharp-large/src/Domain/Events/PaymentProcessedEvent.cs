using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Events;

/// <summary>
/// Domain event raised when a payment is successfully processed.
/// Subscribers may use this to update invoices, send receipts,
/// or trigger downstream accounting workflows.
/// </summary>
public class PaymentProcessedEvent : IDomainEvent
{
    /// <summary>
    /// Gets the unique identifier for this event instance.
    /// </summary>
    public Guid EventId { get; }

    /// <summary>
    /// Gets the UTC timestamp when this event occurred.
    /// </summary>
    public DateTime OccurredAt { get; }

    /// <summary>
    /// Gets the type name of this event.
    /// </summary>
    public string EventType => nameof(PaymentProcessedEvent);

    /// <summary>
    /// Gets the ID of the payment that was processed.
    /// </summary>
    public Guid PaymentId { get; }

    /// <summary>
    /// Gets the ID of the user who made the payment.
    /// </summary>
    public Guid UserId { get; }

    /// <summary>
    /// Gets the payment amount.
    /// </summary>
    public decimal Amount { get; }

    /// <summary>
    /// Gets the currency code for the payment.
    /// </summary>
    public string Currency { get; }

    /// <summary>
    /// Creates a new PaymentProcessedEvent.
    /// </summary>
    public PaymentProcessedEvent(Guid paymentId, Guid userId, decimal amount, string currency)
    {
        EventId = Guid.NewGuid();
        OccurredAt = DateTime.UtcNow;
        PaymentId = paymentId;
        UserId = userId;
        Amount = amount;
        Currency = currency;
    }
}
