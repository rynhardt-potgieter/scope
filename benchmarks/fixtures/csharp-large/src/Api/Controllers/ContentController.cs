using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Application.Mappings;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Exceptions;

namespace CSharpLargeApi.Api.Controllers;

/// <summary>
/// API controller for content management operations.
/// Provides endpoints for content CRUD and publishing workflows.
/// </summary>
public class ContentController
{
    private readonly IContentService _contentService;

    /// <summary>
    /// Initializes the controller with the content service.
    /// </summary>
    public ContentController(IContentService contentService)
    {
        _contentService = contentService ?? throw new ArgumentNullException(nameof(contentService));
    }

    /// <summary>
    /// Creates a new draft content item.
    /// POST /api/content
    /// </summary>
    public async Task<ContentDto> CreateContent(Guid authorId, string title, string slug, string body, CancellationToken cancellationToken = default)
    {
        var content = await _contentService.CreateContentAsync(authorId, title, slug, body, cancellationToken);
        return ContentProfile.ToDto(content);
    }

    /// <summary>
    /// Publishes a content item.
    /// POST /api/content/{id}/publish
    /// </summary>
    public async Task<ContentDto> PublishContent(Guid contentId, CancellationToken cancellationToken = default)
    {
        var content = await _contentService.PublishContentAsync(contentId, cancellationToken);
        return ContentProfile.ToDto(content);
    }

    /// <summary>
    /// Retrieves a content item by its identifier.
    /// GET /api/content/{id}
    /// </summary>
    public async Task<ContentDto> GetContent(Guid contentId, CancellationToken cancellationToken = default)
    {
        var content = await _contentService.GetContentAsync(contentId, cancellationToken);
        if (content is null)
        {
            throw new EntityNotFoundException("Content", contentId);
        }
        return ContentProfile.ToDto(content);
    }

    /// <summary>
    /// Searches content by text with optional status filter.
    /// GET /api/content/search?q={query}&status={status}
    /// </summary>
    public async Task<IReadOnlyList<ContentDto>> SearchContent(string query, ContentStatus? status = null, int skip = 0, int take = 20, CancellationToken cancellationToken = default)
    {
        var results = await _contentService.SearchContentAsync(query, status, skip, take, cancellationToken);
        return ContentProfile.ToDtoList(results);
    }

    /// <summary>
    /// Archives a content item.
    /// POST /api/content/{id}/archive
    /// </summary>
    public async Task<ContentDto> ArchiveContent(Guid contentId, CancellationToken cancellationToken = default)
    {
        var content = await _contentService.ArchiveContentAsync(contentId, cancellationToken);
        return ContentProfile.ToDto(content);
    }
}
