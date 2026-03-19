using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a notification sent to a user through one or more channels.
/// Notifications track their delivery status and support retry logic.
/// </summary>
public class Notification : IAggregateRoot
{
    private readonly List<IDomainEvent> _domainEvents = new();

    /// <summary>
    /// Gets the unique identifier for this notification.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this notification was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this notification was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the ID of the user this notification is addressed to.
    /// </summary>
    public Guid RecipientId { get; private set; }

    /// <summary>
    /// Gets the subject line of this notification.
    /// </summary>
    public string Subject { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the body content of this notification.
    /// </summary>
    public string Body { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the delivery channel for this notification.
    /// </summary>
    public NotificationChannel Channel { get; private set; }

    /// <summary>
    /// Gets whether this notification has been successfully delivered.
    /// </summary>
    public bool IsDelivered { get; private set; }

    /// <summary>
    /// Gets whether this notification has been read by the recipient.
    /// </summary>
    public bool IsRead { get; private set; }

    /// <summary>
    /// Gets the number of delivery attempts made.
    /// </summary>
    public int DeliveryAttempts { get; private set; }

    /// <summary>
    /// Gets the timestamp when this notification was delivered.
    /// </summary>
    public DateTime? DeliveredAt { get; private set; }

    /// <summary>
    /// Gets the collection of domain events raised by this aggregate.
    /// </summary>
    public IReadOnlyCollection<IDomainEvent> DomainEvents => _domainEvents.AsReadOnly();

    /// <summary>
    /// Creates a new notification for the specified recipient.
    /// </summary>
    public static Notification Create(Guid recipientId, string subject, string body, NotificationChannel channel)
    {
        return new Notification
        {
            Id = Guid.NewGuid(),
            RecipientId = recipientId,
            Subject = subject,
            Body = body,
            Channel = channel,
            IsDelivered = false,
            IsRead = false,
            DeliveryAttempts = 0,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Marks this notification as successfully delivered.
    /// </summary>
    public void MarkAsDelivered()
    {
        IsDelivered = true;
        DeliveredAt = DateTime.UtcNow;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Marks this notification as read by the recipient.
    /// </summary>
    public void MarkAsRead()
    {
        IsRead = true;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Increments the delivery attempt counter.
    /// </summary>
    public void IncrementDeliveryAttempts()
    {
        DeliveryAttempts++;
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
