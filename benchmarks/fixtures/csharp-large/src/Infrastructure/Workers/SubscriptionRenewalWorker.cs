using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Infrastructure.Services;

namespace CSharpLargeApi.Infrastructure.Workers;

/// <summary>
/// Background worker that processes subscription renewals on a schedule.
/// Finds subscriptions due for renewal, processes the renewal payment,
/// and updates the subscription status accordingly.
/// Note: This worker calls SubscriptionService.RenewSubscription,
/// NOT ProcessPayment directly. The payment flow goes through
/// the subscription service which does NOT call ProcessPayment either.
/// </summary>
public class SubscriptionRenewalWorker
{
    private readonly SubscriptionService _subscriptionService;
    private readonly IRepository<Subscription> _subscriptionRepository;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the worker with required dependencies.
    /// </summary>
    public SubscriptionRenewalWorker(
        SubscriptionService subscriptionService,
        IRepository<Subscription> subscriptionRepository,
        INotificationService notificationService)
    {
        _subscriptionService = subscriptionService ?? throw new ArgumentNullException(nameof(subscriptionService));
        _subscriptionRepository = subscriptionRepository ?? throw new ArgumentNullException(nameof(subscriptionRepository));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Executes the renewal cycle for all subscriptions due today.
    /// </summary>
    public async Task ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var dueSubscriptions = await _subscriptionService.GetDueForRenewalAsync(
            DateTime.UtcNow, cancellationToken);

        foreach (var subscription in dueSubscriptions)
        {
            if (cancellationToken.IsCancellationRequested)
                break;

            await ProcessRenewal(subscription, cancellationToken);
        }
    }

    /// <summary>
    /// Processes a single subscription renewal.
    /// Renews the subscription and notifies the user.
    /// </summary>
    private async Task ProcessRenewal(Subscription subscription, CancellationToken cancellationToken)
    {
        try
        {
            var renewed = await _subscriptionService.RenewSubscription(
                subscription.Id, cancellationToken);

            await _notificationService.SendEmailAsync(
                subscription.UserId,
                "Subscription Renewed",
                $"Your {subscription.PlanName} subscription has been renewed until {renewed.CurrentBillingPeriod.End:yyyy-MM-dd}.",
                cancellationToken);
        }
        catch (Exception ex)
        {
            // Suspend the subscription on renewal failure
            subscription.Suspend();
            await _subscriptionRepository.SaveChangesAsync(cancellationToken);

            await _notificationService.SendEmailAsync(
                subscription.UserId,
                "Subscription Renewal Failed",
                $"We were unable to renew your {subscription.PlanName} subscription. Please update your payment method. Error: {ex.Message}",
                cancellationToken);
        }
    }
}
