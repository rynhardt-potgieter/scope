using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Events;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a piece of content (article, post, page) in the CMS.
/// Content follows a publishing workflow: Draft -> Review -> Published -> Archived.
/// </summary>
public class Content : IAggregateRoot
{
    private readonly List<IDomainEvent> _domainEvents = new();
    private readonly List<Tag> _tags = new();

    /// <summary>
    /// Gets the unique identifier for this content.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this content was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this content was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the title of this content.
    /// </summary>
    public string Title { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the URL-friendly slug for this content.
    /// </summary>
    public string Slug { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the body text of this content.
    /// </summary>
    public string Body { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the ID of the user who authored this content.
    /// </summary>
    public Guid AuthorId { get; private set; }

    /// <summary>
    /// Gets the current publication status.
    /// </summary>
    public ContentStatus Status { get; private set; }

    /// <summary>
    /// Gets the category this content belongs to.
    /// </summary>
    public Guid? CategoryId { get; private set; }

    /// <summary>
    /// Gets the date this content was published, if applicable.
    /// </summary>
    public DateTime? PublishedAt { get; private set; }

    /// <summary>
    /// Gets the tags associated with this content.
    /// </summary>
    public IReadOnlyList<Tag> Tags => _tags.AsReadOnly();

    /// <summary>
    /// Gets the collection of domain events raised by this aggregate.
    /// </summary>
    public IReadOnlyCollection<IDomainEvent> DomainEvents => _domainEvents.AsReadOnly();

    /// <summary>
    /// Creates a new draft content item.
    /// </summary>
    public static Content Create(Guid authorId, string title, string slug, string body)
    {
        return new Content
        {
            Id = Guid.NewGuid(),
            AuthorId = authorId,
            Title = title,
            Slug = slug,
            Body = body,
            Status = ContentStatus.Draft,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Publishes this content, making it visible to readers.
    /// Raises a ContentPublishedEvent for downstream handlers.
    /// </summary>
    public void Publish()
    {
        Status = ContentStatus.Published;
        PublishedAt = DateTime.UtcNow;
        UpdatedAt = DateTime.UtcNow;

        _domainEvents.Add(new ContentPublishedEvent(Id, Title, AuthorId));
    }

    /// <summary>
    /// Moves this content to review status.
    /// </summary>
    public void SubmitForReview()
    {
        Status = ContentStatus.InReview;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Archives this content, removing it from public view.
    /// </summary>
    public void Archive()
    {
        Status = ContentStatus.Archived;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Assigns a category to this content.
    /// </summary>
    public void AssignCategory(Guid categoryId)
    {
        CategoryId = categoryId;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Adds a tag to this content if not already present.
    /// </summary>
    public void AddTag(Tag tag)
    {
        if (!_tags.Any(t => t.Id == tag.Id))
        {
            _tags.Add(tag);
            UpdatedAt = DateTime.UtcNow;
        }
    }

    /// <summary>
    /// Updates the body text of this content.
    /// </summary>
    public void UpdateBody(string newBody)
    {
        Body = newBody;
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
