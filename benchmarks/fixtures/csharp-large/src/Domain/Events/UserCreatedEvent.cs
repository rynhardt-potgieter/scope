using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Events;

/// <summary>
/// Domain event raised when a new user account is created.
/// Subscribers may use this to send welcome emails, create default
/// profiles, or provision initial resources.
/// </summary>
public class UserCreatedEvent : IDomainEvent
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
    public string EventType => nameof(UserCreatedEvent);

    /// <summary>
    /// Gets the ID of the newly created user.
    /// </summary>
    public Guid UserId { get; }

    /// <summary>
    /// Gets the email address of the newly created user.
    /// </summary>
    public string Email { get; }

    /// <summary>
    /// Creates a new UserCreatedEvent.
    /// </summary>
    public UserCreatedEvent(Guid userId, string email)
    {
        EventId = Guid.NewGuid();
        OccurredAt = DateTime.UtcNow;
        UserId = userId;
        Email = email;
    }
}
