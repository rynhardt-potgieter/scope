using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a tag that can be applied to content items.
/// Tags provide a flat, flexible taxonomy for content discovery.
/// </summary>
public class Tag : IEntity
{
    /// <summary>
    /// Gets the unique identifier for this tag.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this tag was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this tag was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the display name of this tag.
    /// </summary>
    public string Name { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the URL-friendly slug for this tag.
    /// </summary>
    public string Slug { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the number of content items using this tag.
    /// Maintained by the application layer for query efficiency.
    /// </summary>
    public int UsageCount { get; private set; }

    /// <summary>
    /// Creates a new tag with the given name and slug.
    /// </summary>
    public static Tag Create(string name, string slug)
    {
        return new Tag
        {
            Id = Guid.NewGuid(),
            Name = name,
            Slug = slug,
            UsageCount = 0,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Increments the usage count when content is tagged.
    /// </summary>
    public void IncrementUsage()
    {
        UsageCount++;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Decrements the usage count when a tag is removed from content.
    /// </summary>
    public void DecrementUsage()
    {
        if (UsageCount > 0)
        {
            UsageCount--;
        }
        UpdatedAt = DateTime.UtcNow;
    }
}
