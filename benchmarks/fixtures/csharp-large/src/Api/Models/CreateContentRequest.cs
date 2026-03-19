namespace CSharpLargeApi.Api.Models;

/// <summary>
/// Request model for creating new content.
/// Binds to the JSON body of POST /api/content.
/// </summary>
public class CreateContentRequest
{
    /// <summary>Gets or sets the author user ID.</summary>
    public Guid AuthorId { get; set; }

    /// <summary>Gets or sets the content title.</summary>
    public string Title { get; set; } = string.Empty;

    /// <summary>Gets or sets the URL slug.</summary>
    public string Slug { get; set; } = string.Empty;

    /// <summary>Gets or sets the content body.</summary>
    public string Body { get; set; } = string.Empty;

    /// <summary>Gets or sets the optional category ID.</summary>
    public Guid? CategoryId { get; set; }

    /// <summary>Gets or sets optional tag IDs to apply.</summary>
    public List<Guid> TagIds { get; set; } = new();
}
