using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Infrastructure.Persistence.Repositories;

/// <summary>
/// Repository implementation for Content aggregate root persistence.
/// </summary>
public class ContentRepository : IRepository<Content>
{
    private readonly AppDbContext _context;

    /// <summary>
    /// Initializes the repository with the database context.
    /// </summary>
    public ContentRepository(AppDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <summary>
    /// Retrieves a content item by its unique identifier.
    /// </summary>
    public async Task<Content?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Contents.FirstOrDefault(c => c.Id == id);
    }

    /// <summary>
    /// Retrieves all content items.
    /// </summary>
    public async Task<IReadOnlyList<Content>> GetAllAsync(CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Contents.AsReadOnly();
    }

    /// <summary>
    /// Searches content by title or body text with optional status filter.
    /// </summary>
    public async Task<IReadOnlyList<Content>> SearchAsync(string query, ContentStatus? status, int skip, int take, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        var results = _context.Contents.AsEnumerable();

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

        return results
            .OrderByDescending(c => c.CreatedAt)
            .Skip(skip)
            .Take(take)
            .ToList()
            .AsReadOnly();
    }

    /// <summary>
    /// Adds a new content item to the repository.
    /// </summary>
    public async Task<Content> AddAsync(Content entity, CancellationToken cancellationToken = default)
    {
        _context.Contents.Add(entity);
        await _context.SaveChangesAsync(cancellationToken);
        return entity;
    }

    /// <summary>
    /// Updates an existing content item.
    /// </summary>
    public void Update(Content entity)
    {
    }

    /// <summary>
    /// Removes a content item from the repository.
    /// </summary>
    public void Delete(Content entity)
    {
        _context.Contents.Remove(entity);
    }

    /// <summary>
    /// Persists all pending changes.
    /// </summary>
    public async Task<int> SaveChangesAsync(CancellationToken cancellationToken = default)
    {
        return await _context.SaveChangesAsync(cancellationToken);
    }
}
