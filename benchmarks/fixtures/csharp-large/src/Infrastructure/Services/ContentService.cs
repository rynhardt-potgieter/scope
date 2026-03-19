using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Infrastructure.Services;

/// <summary>
/// Implementation of the content service handling content lifecycle operations.
/// </summary>
public class ContentService : IContentService
{
    private readonly IRepository<Content> _contentRepository;

    /// <summary>
    /// Initializes the content service with the content repository.
    /// </summary>
    public ContentService(IRepository<Content> contentRepository)
    {
        _contentRepository = contentRepository ?? throw new ArgumentNullException(nameof(contentRepository));
    }

    /// <summary>
    /// Creates a new draft content item.
    /// </summary>
    public async Task<Content> CreateContentAsync(Guid authorId, string title, string slug, string body, CancellationToken cancellationToken = default)
    {
        var content = Content.Create(authorId, title, slug, body);
        await _contentRepository.AddAsync(content, cancellationToken);
        return content;
    }

    /// <summary>
    /// Publishes a content item, making it publicly visible.
    /// </summary>
    public async Task<Content> PublishContentAsync(Guid contentId, CancellationToken cancellationToken = default)
    {
        var content = await _contentRepository.GetByIdAsync(contentId, cancellationToken);
        if (content is null)
            throw new EntityNotFoundException("Content", contentId);

        content.Publish();
        await _contentRepository.SaveChangesAsync(cancellationToken);
        return content;
    }

    /// <summary>
    /// Archives a content item.
    /// </summary>
    public async Task<Content> ArchiveContentAsync(Guid contentId, CancellationToken cancellationToken = default)
    {
        var content = await _contentRepository.GetByIdAsync(contentId, cancellationToken);
        if (content is null)
            throw new EntityNotFoundException("Content", contentId);

        content.Archive();
        await _contentRepository.SaveChangesAsync(cancellationToken);
        return content;
    }

    /// <summary>
    /// Retrieves a content item by ID.
    /// </summary>
    public async Task<Content?> GetContentAsync(Guid contentId, CancellationToken cancellationToken = default)
    {
        return await _contentRepository.GetByIdAsync(contentId, cancellationToken);
    }

    /// <summary>
    /// Searches content by text with optional status filter.
    /// </summary>
    public async Task<IReadOnlyList<Content>> SearchContentAsync(string query, ContentStatus? status = null, int skip = 0, int take = 20, CancellationToken cancellationToken = default)
    {
        var all = await _contentRepository.GetAllAsync(cancellationToken);
        var results = all.AsEnumerable();

        if (!string.IsNullOrWhiteSpace(query))
        {
            results = results.Where(c =>
                c.Title.Contains(query, StringComparison.OrdinalIgnoreCase) ||
                c.Body.Contains(query, StringComparison.OrdinalIgnoreCase));
        }

        if (status.HasValue)
        {
            results = results.Where(c => c.Status == status.Value);
        }

        return results.OrderByDescending(c => c.CreatedAt).Skip(skip).Take(take).ToList().AsReadOnly();
    }

    /// <summary>
    /// Lists content by author with pagination.
    /// </summary>
    public async Task<IReadOnlyList<Content>> GetContentByAuthorAsync(Guid authorId, int skip = 0, int take = 20, CancellationToken cancellationToken = default)
    {
        var all = await _contentRepository.GetAllAsync(cancellationToken);
        return all.Where(c => c.AuthorId == authorId)
            .OrderByDescending(c => c.CreatedAt)
            .Skip(skip).Take(take)
            .ToList().AsReadOnly();
    }
}
