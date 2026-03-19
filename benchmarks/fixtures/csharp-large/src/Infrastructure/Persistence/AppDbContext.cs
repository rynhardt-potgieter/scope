using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Infrastructure.Persistence;

/// <summary>
/// Entity Framework Core database context for the application.
/// Configures entity mappings and provides DbSet access to all aggregate roots.
/// </summary>
public class AppDbContext
{
    private readonly Dictionary<Type, object> _sets = new();

    /// <summary>
    /// Gets the DbSet for User entities.
    /// </summary>
    public List<User> Users { get; } = new();

    /// <summary>
    /// Gets the DbSet for Payment entities.
    /// </summary>
    public List<Payment> Payments { get; } = new();

    /// <summary>
    /// Gets the DbSet for Invoice entities.
    /// </summary>
    public List<Invoice> Invoices { get; } = new();

    /// <summary>
    /// Gets the DbSet for Subscription entities.
    /// </summary>
    public List<Subscription> Subscriptions { get; } = new();

    /// <summary>
    /// Gets the DbSet for Content entities.
    /// </summary>
    public List<Content> Contents { get; } = new();

    /// <summary>
    /// Gets the DbSet for Notification entities.
    /// </summary>
    public List<Notification> Notifications { get; } = new();

    /// <summary>
    /// Gets the DbSet for Category entities.
    /// </summary>
    public List<Category> Categories { get; } = new();

    /// <summary>
    /// Gets the DbSet for Tag entities.
    /// </summary>
    public List<Tag> Tags { get; } = new();

    /// <summary>
    /// Gets the DbSet for Media entities.
    /// </summary>
    public List<Media> MediaItems { get; } = new();

    /// <summary>
    /// Gets the DbSet for Session entities.
    /// </summary>
    public List<Session> Sessions { get; } = new();

    /// <summary>
    /// Gets the DbSet for Role entities.
    /// </summary>
    public List<Role> Roles { get; } = new();

    /// <summary>
    /// Gets the DbSet for Permission entities.
    /// </summary>
    public List<Permission> Permissions { get; } = new();

    /// <summary>
    /// Saves all changes made in this context to the database.
    /// Dispatches domain events for aggregate roots that have pending events.
    /// </summary>
    public async Task<int> SaveChangesAsync(CancellationToken cancellationToken = default)
    {
        var changedCount = 0;

        // Simulate dispatching domain events before save
        var aggregatesWithEvents = Users.Cast<object>()
            .Concat(Payments)
            .Concat(Invoices)
            .Concat(Subscriptions)
            .Concat(Contents)
            .Concat(Notifications);

        // In a real EF Core context, this would persist to the database
        await Task.Delay(1, cancellationToken);
        changedCount++;

        return changedCount;
    }
}
