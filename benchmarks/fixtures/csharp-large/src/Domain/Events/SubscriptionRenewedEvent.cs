using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Events;

/// <summary>
/// Domain event raised when a subscription is successfully renewed.
/// Subscribers may use this to send renewal confirmations, update
/// billing records, or extend feature access.
/// </summary>
public class SubscriptionRenewedEvent : IDomainEvent
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
    public string EventType => nameof(SubscriptionRenewedEvent);

    /// <summary>
    /// Gets the ID of the renewed subscription.
    /// </summary>
    public Guid SubscriptionId { get; }

    /// <summary>
    /// Gets the ID of the user who owns the subscription.
    /// </summary>
    public Guid UserId { get; }

    /// <summary>
    /// Gets the plan name of the renewed subscription.
    /// </summary>
    public string PlanName { get; }

    /// <summary>
    /// Creates a new SubscriptionRenewedEvent.
    /// </summary>
    public SubscriptionRenewedEvent(Guid subscriptionId, Guid userId, string planName)
    {
        EventId = Guid.NewGuid();
        OccurredAt = DateTime.UtcNow;
        SubscriptionId = subscriptionId;
        UserId = userId;
        PlanName = planName;
    }
}
