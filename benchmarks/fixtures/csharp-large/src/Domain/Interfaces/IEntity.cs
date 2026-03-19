namespace CSharpLargeApi.Domain.Interfaces;

/// <summary>
/// Marker interface for all domain entities.
/// Provides a common identity contract across the domain model.
/// </summary>
public interface IEntity
{
    /// <summary>
    /// Gets the unique identifier for this entity.
    /// </summary>
    Guid Id { get; }

    /// <summary>
    /// Gets the timestamp when this entity was created.
    /// </summary>
    DateTime CreatedAt { get; }

    /// <summary>
    /// Gets the timestamp when this entity was last modified.
    /// </summary>
    DateTime? UpdatedAt { get; }
}
