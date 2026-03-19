using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Events;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a recurring subscription plan for a user.
/// Manages billing cycles, renewal dates, and plan upgrades/downgrades.
/// </summary>
public class Subscription : IAggregateRoot
{
    private readonly List<IDomainEvent> _domainEvents = new();

    /// <summary>
    /// Gets the unique identifier for this subscription.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this subscription was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this subscription was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the ID of the user who owns this subscription.
    /// </summary>
    public Guid UserId { get; private set; }

    /// <summary>
    /// Gets the name of the subscription plan (e.g. "Pro", "Enterprise").
    /// </summary>
    public string PlanName { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the recurring price for this subscription.
    /// </summary>
    public Money Price { get; private set; } = null!;

    /// <summary>
    /// Gets the current status of this subscription.
    /// </summary>
    public SubscriptionStatus Status { get; private set; }

    /// <summary>
    /// Gets the billing period for this subscription.
    /// </summary>
    public DateRange CurrentBillingPeriod { get; private set; } = null!;

    /// <summary>
    /// Gets the date when this subscription next renews.
    /// </summary>
    public DateTime NextRenewalDate { get; private set; }

    /// <summary>
    /// Gets the date when this subscription was cancelled, if applicable.
    /// </summary>
    public DateTime? CancelledAt { get; private set; }

    /// <summary>
    /// Gets the collection of domain events raised by this aggregate.
    /// </summary>
    public IReadOnlyCollection<IDomainEvent> DomainEvents => _domainEvents.AsReadOnly();

    /// <summary>
    /// Creates a new active subscription for a user.
    /// </summary>
    public static Subscription Create(Guid userId, string planName, Money price, DateRange billingPeriod)
    {
        return new Subscription
        {
            Id = Guid.NewGuid(),
            UserId = userId,
            PlanName = planName,
            Price = price,
            Status = SubscriptionStatus.Active,
            CurrentBillingPeriod = billingPeriod,
            NextRenewalDate = billingPeriod.End,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Renews this subscription for the next billing period.
    /// Raises a SubscriptionRenewedEvent for downstream processing.
    /// </summary>
    public void Renew(DateRange newBillingPeriod)
    {
        CurrentBillingPeriod = newBillingPeriod;
        NextRenewalDate = newBillingPeriod.End;
        Status = SubscriptionStatus.Active;
        UpdatedAt = DateTime.UtcNow;

        _domainEvents.Add(new SubscriptionRenewedEvent(Id, UserId, PlanName));
    }

    /// <summary>
    /// Cancels this subscription effective at the end of the current period.
    /// </summary>
    public void Cancel()
    {
        Status = SubscriptionStatus.Cancelled;
        CancelledAt = DateTime.UtcNow;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Suspends this subscription due to payment failure.
    /// </summary>
    public void Suspend()
    {
        Status = SubscriptionStatus.Suspended;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Upgrades the subscription to a new plan with a different price.
    /// </summary>
    public void UpgradePlan(string newPlanName, Money newPrice)
    {
        PlanName = newPlanName;
        Price = newPrice;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Clears all pending domain events from this aggregate.
    /// </summary>
    public void ClearDomainEvents()
    {
        _domainEvents.Clear();
    }
}
