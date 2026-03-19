using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Events;

/// <summary>
/// Domain event raised when a content item is published.
/// Subscribers may use this to invalidate caches, send notifications
/// to followers, or update search indices.
/// </summary>
public class ContentPublishedEvent : IDomainEvent
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
    public string EventType => nameof(ContentPublishedEvent);

    /// <summary>
    /// Gets the ID of the published content.
    /// </summary>
    public Guid ContentId { get; }

    /// <summary>
    /// Gets the title of the published content.
    /// </summary>
    public string Title { get; }

    /// <summary>
    /// Gets the ID of the author who published the content.
    /// </summary>
    public Guid AuthorId { get; }

    /// <summary>
    /// Creates a new ContentPublishedEvent.
    /// </summary>
    public ContentPublishedEvent(Guid contentId, string title, Guid authorId)
    {
        EventId = Guid.NewGuid();
        OccurredAt = DateTime.UtcNow;
        ContentId = contentId;
        Title = title;
        AuthorId = authorId;
    }
}
