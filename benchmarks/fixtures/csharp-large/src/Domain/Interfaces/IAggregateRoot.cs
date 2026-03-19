namespace CSharpLargeApi.Domain.Interfaces;

/// <summary>
/// Marker interface for aggregate root entities.
/// Aggregate roots are the only entities that can be directly retrieved
/// from repositories and serve as consistency boundaries.
/// </summary>
public interface IAggregateRoot : IEntity
{
    /// <summary>
    /// Gets the collection of domain events raised by this aggregate.
    /// Events are cleared after being dispatched by the infrastructure layer.
    /// </summary>
    IReadOnlyCollection<IDomainEvent> DomainEvents { get; }

    /// <summary>
    /// Clears all pending domain events from this aggregate.
    /// Called by the infrastructure after successful event dispatch.
    /// </summary>
    void ClearDomainEvents();
}
