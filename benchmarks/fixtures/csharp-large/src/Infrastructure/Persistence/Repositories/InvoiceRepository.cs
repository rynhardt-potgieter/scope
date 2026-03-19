using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Infrastructure.Persistence.Repositories;

/// <summary>
/// Repository implementation for Invoice aggregate root persistence.
/// </summary>
public class InvoiceRepository : IRepository<Invoice>
{
    private readonly AppDbContext _context;

    /// <summary>
    /// Initializes the repository with the database context.
    /// </summary>
    public InvoiceRepository(AppDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <summary>
    /// Retrieves an invoice by its unique identifier.
    /// </summary>
    public async Task<Invoice?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Invoices.FirstOrDefault(i => i.Id == id);
    }

    /// <summary>
    /// Retrieves all invoices.
    /// </summary>
    public async Task<IReadOnlyList<Invoice>> GetAllAsync(CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Invoices.AsReadOnly();
    }

    /// <summary>
    /// Retrieves invoices for a specific user.
    /// </summary>
    public async Task<IReadOnlyList<Invoice>> GetByUserIdAsync(Guid userId, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Invoices
            .Where(i => i.UserId == userId)
            .OrderByDescending(i => i.CreatedAt)
            .ToList()
            .AsReadOnly();
    }

    /// <summary>
    /// Retrieves unsettled invoices that are past due.
    /// </summary>
    public async Task<IReadOnlyList<Invoice>> GetOverdueAsync(CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Invoices
            .Where(i => !i.IsSettled && i.DueDate < DateTime.UtcNow)
            .OrderBy(i => i.DueDate)
            .ToList()
            .AsReadOnly();
    }

    /// <summary>
    /// Adds a new invoice to the repository.
    /// </summary>
    public async Task<Invoice> AddAsync(Invoice entity, CancellationToken cancellationToken = default)
    {
        _context.Invoices.Add(entity);
        await _context.SaveChangesAsync(cancellationToken);
        return entity;
    }

    /// <summary>
    /// Updates an existing invoice.
    /// </summary>
    public void Update(Invoice entity)
    {
    }

    /// <summary>
    /// Removes an invoice from the repository.
    /// </summary>
    public void Delete(Invoice entity)
    {
        _context.Invoices.Remove(entity);
    }

    /// <summary>
    /// Persists all pending changes.
    /// </summary>
    public async Task<int> SaveChangesAsync(CancellationToken cancellationToken = default)
    {
        return await _context.SaveChangesAsync(cancellationToken);
    }
}
