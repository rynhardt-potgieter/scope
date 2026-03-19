using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a content category used to organize articles and pages.
/// Categories can be nested in a parent-child hierarchy.
/// </summary>
public class Category : IEntity
{
    /// <summary>
    /// Gets the unique identifier for this category.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this category was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this category was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the display name of this category.
    /// </summary>
    public string Name { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the URL-friendly slug for this category.
    /// </summary>
    public string Slug { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the optional parent category ID for hierarchical nesting.
    /// </summary>
    public Guid? ParentId { get; private set; }

    /// <summary>
    /// Gets the sort order for displaying this category among siblings.
    /// </summary>
    public int SortOrder { get; private set; }

    /// <summary>
    /// Creates a new category with the given name and slug.
    /// </summary>
    public static Category Create(string name, string slug, Guid? parentId = null)
    {
        return new Category
        {
            Id = Guid.NewGuid(),
            Name = name,
            Slug = slug,
            ParentId = parentId,
            SortOrder = 0,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Updates the category's display name and slug.
    /// </summary>
    public void UpdateName(string name, string slug)
    {
        Name = name;
        Slug = slug;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Moves this category to a different parent.
    /// </summary>
    public void MoveToParent(Guid? newParentId)
    {
        ParentId = newParentId;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Sets the sort order for this category.
    /// </summary>
    public void SetSortOrder(int order)
    {
        SortOrder = order;
        UpdatedAt = DateTime.UtcNow;
    }
}
