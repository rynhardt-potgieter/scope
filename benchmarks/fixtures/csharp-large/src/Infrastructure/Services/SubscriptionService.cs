using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Infrastructure.Services;

/// <summary>
/// Service responsible for subscription lifecycle management.
/// Handles creation, renewal, cancellation, and plan changes.
/// Note: This service does NOT directly call ProcessPayment.
/// Subscription renewal payments are triggered by the SubscriptionController
/// or the SubscriptionRenewalWorker, which call ProcessPayment themselves.
/// </summary>
public class SubscriptionService
{
    private readonly IRepository<Subscription> _subscriptionRepository;
    private readonly IUserService _userService;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the subscription service with required dependencies.
    /// </summary>
    public SubscriptionService(
        IRepository<Subscription> subscriptionRepository,
        IUserService userService,
        INotificationService notificationService)
    {
        _subscriptionRepository = subscriptionRepository ?? throw new ArgumentNullException(nameof(subscriptionRepository));
        _userService = userService ?? throw new ArgumentNullException(nameof(userService));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Creates a new subscription for a user.
    /// </summary>
    public async Task<Subscription> CreateSubscription(Guid userId, string planName, Money price, CancellationToken cancellationToken = default)
    {
        var user = await _userService.GetUserAsync(userId, cancellationToken);
        if (user is null)
        {
            throw new EntityNotFoundException("User", userId);
        }

        var billingStart = DateTime.UtcNow;
        var billingEnd = billingStart.AddMonths(1).AddDays(-1);
        var billingPeriod = new DateRange(billingStart, billingEnd);

        var subscription = Subscription.Create(userId, planName, price, billingPeriod);

        await _subscriptionRepository.AddAsync(subscription, cancellationToken);

        await _notificationService.SendEmailAsync(
            userId,
            "Subscription Created",
            $"Your {planName} subscription has been activated at {price}/month.",
            cancellationToken);

        return subscription;
    }

    /// <summary>
    /// Renews a subscription for the next billing period.
    /// This method only handles the subscription state change.
    /// The payment is processed separately by the caller.
    /// </summary>
    public async Task<Subscription> RenewSubscription(Guid subscriptionId, CancellationToken cancellationToken = default)
    {
        var subscription = await _subscriptionRepository.GetByIdAsync(subscriptionId, cancellationToken);
        if (subscription is null)
        {
            throw new EntityNotFoundException("Subscription", subscriptionId);
        }

        if (subscription.Status == SubscriptionStatus.Cancelled)
        {
            throw new BusinessRuleException(
                "ActiveSubscriptionRequired",
                "Cannot renew a cancelled subscription.");
        }

        var newBillingPeriod = subscription.CurrentBillingPeriod.NextMonth();
        subscription.Renew(newBillingPeriod);

        await _subscriptionRepository.SaveChangesAsync(cancellationToken);

        return subscription;
    }

    /// <summary>
    /// Cancels a subscription effective at the end of the current billing period.
    /// </summary>
    public async Task<Subscription> CancelSubscription(Guid subscriptionId, CancellationToken cancellationToken = default)
    {
        var subscription = await _subscriptionRepository.GetByIdAsync(subscriptionId, cancellationToken);
        if (subscription is null)
        {
            throw new EntityNotFoundException("Subscription", subscriptionId);
        }

        subscription.Cancel();
        await _subscriptionRepository.SaveChangesAsync(cancellationToken);

        await _notificationService.SendEmailAsync(
            subscription.UserId,
            "Subscription Cancelled",
            $"Your {subscription.PlanName} subscription has been cancelled. Access continues until {subscription.CurrentBillingPeriod.End:yyyy-MM-dd}.",
            cancellationToken);

        return subscription;
    }

    /// <summary>
    /// Retrieves a subscription by ID.
    /// </summary>
    public async Task<Subscription?> GetSubscriptionAsync(Guid subscriptionId, CancellationToken cancellationToken = default)
    {
        return await _subscriptionRepository.GetByIdAsync(subscriptionId, cancellationToken);
    }

    /// <summary>
    /// Retrieves all subscriptions due for renewal before the given date.
    /// </summary>
    public async Task<IReadOnlyList<Subscription>> GetDueForRenewalAsync(DateTime cutoffDate, CancellationToken cancellationToken = default)
    {
        var all = await _subscriptionRepository.GetAllAsync(cancellationToken);
        return all
            .Where(s => s.Status == SubscriptionStatus.Active && s.NextRenewalDate <= cutoffDate)
            .OrderBy(s => s.NextRenewalDate)
            .ToList()
            .AsReadOnly();
    }
}
