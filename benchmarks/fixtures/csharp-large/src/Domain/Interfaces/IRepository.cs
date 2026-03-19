namespace CSharpLargeApi.Domain.Interfaces;

/// <summary>
/// Generic repository interface for aggregate root persistence.
/// Implementations live in the Infrastructure layer and use EF Core
/// or another ORM to fulfil the contract.
/// </summary>
/// <typeparam name="T">The aggregate root entity type.</typeparam>
public interface IRepository<T> where T : class, IAggregateRoot
{
    /// <summary>
    /// Retrieves an entity by its unique identifier.
    /// Returns null if no entity is found with the given ID.
    /// </summary>
    Task<T?> GetByIdAsync(Guid id, CancellationToken cancellationToken = default);

    /// <summary>
    /// Retrieves all entities of this type.
    /// Use sparingly — prefer filtered queries for large datasets.
    /// </summary>
    Task<IReadOnlyList<T>> GetAllAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Adds a new entity to the repository.
    /// The entity is not persisted until SaveChangesAsync is called on the unit of work.
    /// </summary>
    Task<T> AddAsync(T entity, CancellationToken cancellationToken = default);

    /// <summary>
    /// Updates an existing entity in the repository.
    /// </summary>
    void Update(T entity);

    /// <summary>
    /// Removes an entity from the repository.
    /// </summary>
    void Delete(T entity);

    /// <summary>
    /// Persists all pending changes to the underlying store.
    /// </summary>
    Task<int> SaveChangesAsync(CancellationToken cancellationToken = default);
}
