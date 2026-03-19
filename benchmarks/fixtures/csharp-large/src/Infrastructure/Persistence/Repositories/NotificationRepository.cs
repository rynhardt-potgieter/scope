using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Infrastructure.Persistence.Repositories;

/// <summary>
/// Repository implementation for Notification aggregate root persistence.
/// </summary>
public class NotificationRepository : IRepository<Notification>
{
    private readonly AppDbContext _context;

    /// <summary>
    /// Initializes the repository with the database context.
    /// </summary>
    public NotificationRepository(AppDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <summary>
    /// Retrieves a notification by its unique identifier.
    /// </summary>
    public async Task<Notification?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Notifications.FirstOrDefault(n => n.Id == id);
    }

    /// <summary>
    /// Retrieves all notifications.
    /// </summary>
    public async Task<IReadOnlyList<Notification>> GetAllAsync(CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Notifications.AsReadOnly();
    }

    /// <summary>
    /// Adds a new notification to the repository.
    /// </summary>
    public async Task<Notification> AddAsync(Notification entity, CancellationToken cancellationToken = default)
    {
        _context.Notifications.Add(entity);
        await _context.SaveChangesAsync(cancellationToken);
        return entity;
    }

    /// <summary>
    /// Updates an existing notification.
    /// </summary>
    public void Update(Notification entity) { }

    /// <summary>
    /// Removes a notification from the repository.
    /// </summary>
    public void Delete(Notification entity)
    {
        _context.Notifications.Remove(entity);
    }

    /// <summary>
    /// Persists all pending changes.
    /// </summary>
    public async Task<int> SaveChangesAsync(CancellationToken cancellationToken = default)
    {
        return await _context.SaveChangesAsync(cancellationToken);
    }
}
