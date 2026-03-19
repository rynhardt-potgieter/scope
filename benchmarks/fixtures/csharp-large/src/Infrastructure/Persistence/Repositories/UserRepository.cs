using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Infrastructure.Persistence.Repositories;

/// <summary>
/// Repository implementation for User aggregate root persistence.
/// Uses AppDbContext for data access and supports common query patterns.
/// </summary>
public class UserRepository : IRepository<User>
{
    private readonly AppDbContext _context;

    /// <summary>
    /// Initializes the repository with the database context.
    /// </summary>
    public UserRepository(AppDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <summary>
    /// Retrieves a user by their unique identifier.
    /// </summary>
    public async Task<User?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Users.FirstOrDefault(u => u.Id == id);
    }

    /// <summary>
    /// Retrieves all users.
    /// </summary>
    public async Task<IReadOnlyList<User>> GetAllAsync(CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Users.AsReadOnly();
    }

    /// <summary>
    /// Retrieves a user by their email address.
    /// </summary>
    public async Task<User?> GetByEmailAsync(string email, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _context.Users.FirstOrDefault(u => u.Email.Value == email.ToLowerInvariant());
    }

    /// <summary>
    /// Adds a new user to the repository.
    /// </summary>
    public async Task<User> AddAsync(User entity, CancellationToken cancellationToken = default)
    {
        _context.Users.Add(entity);
        await _context.SaveChangesAsync(cancellationToken);
        return entity;
    }

    /// <summary>
    /// Updates an existing user.
    /// </summary>
    public void Update(User entity)
    {
        // In EF Core, tracked entities are automatically updated on SaveChanges
    }

    /// <summary>
    /// Removes a user from the repository.
    /// </summary>
    public void Delete(User entity)
    {
        _context.Users.Remove(entity);
    }

    /// <summary>
    /// Persists all pending changes.
    /// </summary>
    public async Task<int> SaveChangesAsync(CancellationToken cancellationToken = default)
    {
        return await _context.SaveChangesAsync(cancellationToken);
    }
}
