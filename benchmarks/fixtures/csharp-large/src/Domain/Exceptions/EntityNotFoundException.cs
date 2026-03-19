namespace CSharpLargeApi.Domain.Exceptions;

/// <summary>
/// Exception thrown when a requested entity cannot be found.
/// Typically maps to HTTP 404 at the API layer.
/// </summary>
public class EntityNotFoundException : DomainException
{
    /// <summary>
    /// Gets the type name of the entity that was not found.
    /// </summary>
    public string EntityType { get; }

    /// <summary>
    /// Gets the identifier that was used to look up the entity.
    /// </summary>
    public string EntityId { get; }

    /// <summary>
    /// Creates a new EntityNotFoundException for the given entity type and ID.
    /// </summary>
    public EntityNotFoundException(string entityType, string entityId)
        : base($"{entityType} with ID '{entityId}' was not found.", "ENTITY_NOT_FOUND")
    {
        EntityType = entityType;
        EntityId = entityId;
    }

    /// <summary>
    /// Creates a new EntityNotFoundException for the given entity type and GUID.
    /// </summary>
    public EntityNotFoundException(string entityType, Guid entityId)
        : this(entityType, entityId.ToString())
    {
    }

    /// <summary>
    /// Creates an EntityNotFoundException using the generic type parameter for the entity name.
    /// </summary>
    public static EntityNotFoundException For<T>(Guid id) where T : class
    {
        return new EntityNotFoundException(typeof(T).Name, id);
    }
}
