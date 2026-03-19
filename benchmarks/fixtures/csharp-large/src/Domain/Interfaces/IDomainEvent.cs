namespace CSharpLargeApi.Domain.Interfaces;

/// <summary>
/// Represents a domain event that signals something meaningful
/// happened within the domain. Events are dispatched after the
/// aggregate root is persisted.
/// </summary>
public interface IDomainEvent
{
    /// <summary>
    /// Gets the unique identifier for this event instance.
    /// Used for idempotency checks in event handlers.
    /// </summary>
    Guid EventId { get; }

    /// <summary>
    /// Gets the UTC timestamp when this event occurred.
    /// </summary>
    DateTime OccurredAt { get; }

    /// <summary>
    /// Gets the type name of this event for routing and serialization.
    /// </summary>
    string EventType { get; }
}
