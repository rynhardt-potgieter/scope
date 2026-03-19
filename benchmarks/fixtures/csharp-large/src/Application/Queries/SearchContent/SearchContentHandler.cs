using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;

namespace CSharpLargeApi.Application.Queries.SearchContent;

/// <summary>
/// Handles the SearchContentQuery by delegating to the content service
/// and mapping results to DTOs.
/// </summary>
public class SearchContentHandler
{
    private readonly IContentService _contentService;

    /// <summary>
    /// Initializes the handler with the content service dependency.
    /// </summary>
    public SearchContentHandler(IContentService contentService)
    {
        _contentService = contentService ?? throw new ArgumentNullException(nameof(contentService));
    }

    /// <summary>
    /// Handles the query by searching content and mapping results.
    /// </summary>
    public async Task<IReadOnlyList<ContentDto>> Handle(SearchContentQuery query, CancellationToken cancellationToken)
    {
        var results = await _contentService.SearchContentAsync(
            query.SearchText, query.Status, query.Skip, query.Take, cancellationToken);

        return results.Select(c => new ContentDto
        {
            Id = c.Id,
            Title = c.Title,
            Slug = c.Slug,
            AuthorId = c.AuthorId,
            Status = c.Status.ToString(),
            PublishedAt = c.PublishedAt,
            CreatedAt = c.CreatedAt
        }).ToList().AsReadOnly();
    }
}
