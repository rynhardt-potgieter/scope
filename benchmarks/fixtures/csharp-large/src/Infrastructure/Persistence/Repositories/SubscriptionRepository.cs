using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Infrastructure.Persistence.Repositories;

/// <summary>
/// Repository implementation for Subscription aggregate root persistence.
/// </summary>
public class SubscriptionRepository : IRepository<Subscription>
{
    private readonly AppDbContext _context;

    /// <summary>
    /// Initializes the repository with the database context.
    /// </summary>
    public SubscriptionRepository(AppDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <summary>
    /// Retrieves a subscription by its unique identifier.
    /// </summary>
    public async Task<Subscription?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Subscriptions.FirstOrDefault(s => s.Id == id);
    }

    /// <summary>
    /// Retrieves all subscriptions.
    /// </summary>
    public async Task<IReadOnlyList<Subscription>> GetAllAsync(CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Subscriptions.AsReadOnly();
    }

    /// <summary>
    /// Retrieves subscriptions due for renewal.
    /// </summary>
    public async Task<IReadOnlyList<Subscription>> GetDueForRenewalAsync(DateTime cutoffDate, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Subscriptions
            .Where(s => s.Status == SubscriptionStatus.Active && s.NextRenewalDate <= cutoffDate)
            .OrderBy(s => s.NextRenewalDate)
            .ToList()
            .AsReadOnly();
    }

    /// <summary>
    /// Adds a new subscription to the repository.
    /// </summary>
    public async Task<Subscription> AddAsync(Subscription entity, CancellationToken cancellationToken = default)
    {
        _context.Subscriptions.Add(entity);
        await _context.SaveChangesAsync(cancellationToken);
        return entity;
    }

    /// <summary>
    /// Updates an existing subscription.
    /// </summary>
    public void Update(Subscription entity)
    {
    }

    /// <summary>
    /// Removes a subscription from the repository.
    /// </summary>
    public void Delete(Subscription entity)
    {
        _context.Subscriptions.Remove(entity);
    }

    /// <summary>
    /// Persists all pending changes.
    /// </summary>
    public async Task<int> SaveChangesAsync(CancellationToken cancellationToken = default)
    {
        return await _context.SaveChangesAsync(cancellationToken);
    }
}
