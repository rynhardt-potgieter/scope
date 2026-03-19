using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Events;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a financial payment transaction in the system.
/// Tracks the full lifecycle from creation through settlement or failure.
/// </summary>
public class Payment : IAggregateRoot
{
    private readonly List<IDomainEvent> _domainEvents = new();

    /// <summary>
    /// Gets the unique identifier for this payment.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this payment was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this payment was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the monetary amount and currency for this payment.
    /// </summary>
    public Money Amount { get; private set; } = null!;

    /// <summary>
    /// Gets the current status of this payment.
    /// </summary>
    public PaymentStatus Status { get; private set; }

    /// <summary>
    /// Gets the ID of the user who initiated this payment.
    /// </summary>
    public Guid UserId { get; private set; }

    /// <summary>
    /// Gets the external transaction reference from the payment gateway.
    /// </summary>
    public string? GatewayTransactionId { get; private set; }

    /// <summary>
    /// Gets the error message if the payment failed.
    /// </summary>
    public string? FailureReason { get; private set; }

    /// <summary>
    /// Gets the number of retry attempts made for this payment.
    /// </summary>
    public int RetryCount { get; private set; }

    /// <summary>
    /// Gets the collection of domain events raised by this aggregate.
    /// </summary>
    public IReadOnlyCollection<IDomainEvent> DomainEvents => _domainEvents.AsReadOnly();

    /// <summary>
    /// Creates a new pending payment for the specified user and amount.
    /// </summary>
    public static Payment Create(Guid userId, Money amount)
    {
        return new Payment
        {
            Id = Guid.NewGuid(),
            UserId = userId,
            Amount = amount,
            Status = PaymentStatus.Pending,
            RetryCount = 0,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Marks this payment as successfully processed.
    /// Raises a PaymentProcessedEvent for downstream handlers.
    /// </summary>
    public void MarkAsProcessed(string gatewayTransactionId)
    {
        Status = PaymentStatus.Completed;
        GatewayTransactionId = gatewayTransactionId;
        UpdatedAt = DateTime.UtcNow;

        _domainEvents.Add(new PaymentProcessedEvent(Id, UserId, Amount.Amount, Amount.Currency));
    }

    /// <summary>
    /// Marks this payment as failed with the given reason.
    /// </summary>
    public void MarkAsFailed(string reason)
    {
        Status = PaymentStatus.Failed;
        FailureReason = reason;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Marks this payment as refunded.
    /// </summary>
    public void MarkAsRefunded()
    {
        Status = PaymentStatus.Refunded;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Increments the retry counter for failed payment reattempts.
    /// </summary>
    public void IncrementRetryCount()
    {
        RetryCount++;
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
