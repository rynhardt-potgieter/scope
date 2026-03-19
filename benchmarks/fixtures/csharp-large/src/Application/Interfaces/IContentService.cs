using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;

namespace CSharpLargeApi.Application.Interfaces;

/// <summary>
/// Defines the contract for content management operations.
/// Handles creation, publishing workflow, and content queries.
/// </summary>
public interface IContentService
{
    /// <summary>
    /// Creates a new draft content item.
    /// </summary>
    Task<Content> CreateContentAsync(Guid authorId, string title, string slug, string body, CancellationToken cancellationToken = default);

    /// <summary>
    /// Publishes a content item, making it visible to readers.
    /// </summary>
    Task<Content> PublishContentAsync(Guid contentId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Archives a content item, removing it from public view.
    /// </summary>
    Task<Content> ArchiveContentAsync(Guid contentId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Retrieves a content item by its unique identifier.
    /// </summary>
    Task<Content?> GetContentAsync(Guid contentId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Searches content by title or body text.
    /// </summary>
    Task<IReadOnlyList<Content>> SearchContentAsync(string query, ContentStatus? status = null, int skip = 0, int take = 20, CancellationToken cancellationToken = default);

    /// <summary>
    /// Lists content by author with pagination.
    /// </summary>
    Task<IReadOnlyList<Content>> GetContentByAuthorAsync(Guid authorId, int skip = 0, int take = 20, CancellationToken cancellationToken = default);
}
