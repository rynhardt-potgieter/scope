namespace CSharpLargeApi.Application.DTOs;

/// <summary>
/// Data transfer object for content information.
/// Provides a summary view without the full body text.
/// </summary>
public class ContentDto
{
    /// <summary>
    /// Gets or sets the content identifier.
    /// </summary>
    public Guid Id { get; set; }

    /// <summary>
    /// Gets or sets the content title.
    /// </summary>
    public string Title { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the URL slug.
    /// </summary>
    public string Slug { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the author identifier.
    /// </summary>
    public Guid AuthorId { get; set; }

    /// <summary>
    /// Gets or sets the publication status.
    /// </summary>
    public string Status { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the category identifier, if assigned.
    /// </summary>
    public Guid? CategoryId { get; set; }

    /// <summary>
    /// Gets or sets the publication timestamp.
    /// </summary>
    public DateTime? PublishedAt { get; set; }

    /// <summary>
    /// Gets or sets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; set; }
}
