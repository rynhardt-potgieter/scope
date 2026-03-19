using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Infrastructure.Persistence.Repositories;

/// <summary>
/// Repository implementation for Payment aggregate root persistence.
/// Provides payment-specific query methods beyond the generic interface.
/// </summary>
public class PaymentRepository : IRepository<Payment>
{
    private readonly AppDbContext _context;

    /// <summary>
    /// Initializes the repository with the database context.
    /// </summary>
    public PaymentRepository(AppDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <summary>
    /// Retrieves a payment by its unique identifier.
    /// </summary>
    public async Task<Payment?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Payments.FirstOrDefault(p => p.Id == id);
    }

    /// <summary>
    /// Retrieves all payments.
    /// </summary>
    public async Task<IReadOnlyList<Payment>> GetAllAsync(CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Payments.AsReadOnly();
    }

    /// <summary>
    /// Retrieves payments for a specific user with pagination.
    /// </summary>
    public async Task<IReadOnlyList<Payment>> GetByUserIdAsync(Guid userId, int skip, int take, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Payments
            .Where(p => p.UserId == userId)
            .OrderByDescending(p => p.CreatedAt)
            .Skip(skip)
            .Take(take)
            .ToList()
            .AsReadOnly();
    }

    /// <summary>
    /// Retrieves payments with a specific status for retry processing.
    /// </summary>
    public async Task<IReadOnlyList<Payment>> GetByStatusAsync(PaymentStatus status, int maxResults, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Payments
            .Where(p => p.Status == status)
            .OrderBy(p => p.CreatedAt)
            .Take(maxResults)
            .ToList()
            .AsReadOnly();
    }

    /// <summary>
    /// Adds a new payment to the repository.
    /// </summary>
    public async Task<Payment> AddAsync(Payment entity, CancellationToken cancellationToken = default)
    {
        _context.Payments.Add(entity);
        await _context.SaveChangesAsync(cancellationToken);
        return entity;
    }

    /// <summary>
    /// Updates an existing payment.
    /// </summary>
    public void Update(Payment entity)
    {
        // Tracked by EF Core change tracker
    }

    /// <summary>
    /// Removes a payment from the repository.
    /// </summary>
    public void Delete(Payment entity)
    {
        _context.Payments.Remove(entity);
    }

    /// <summary>
    /// Persists all pending changes.
    /// </summary>
    public async Task<int> SaveChangesAsync(CancellationToken cancellationToken = default)
    {
        return await _context.SaveChangesAsync(cancellationToken);
    }
}
