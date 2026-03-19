using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Events;

namespace CSharpLargeApi.Application.Events;

/// <summary>
/// Handles SubscriptionRenewedEvent by sending a renewal confirmation
/// and updating subscription caches.
/// </summary>
public class SubscriptionRenewedEventHandler
{
    private readonly INotificationService _notificationService;
    private readonly ICacheService _cacheService;

    /// <summary>
    /// Initializes the handler with required dependencies.
    /// </summary>
    public SubscriptionRenewedEventHandler(
        INotificationService notificationService,
        ICacheService cacheService)
    {
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
        _cacheService = cacheService ?? throw new ArgumentNullException(nameof(cacheService));
    }

    /// <summary>
    /// Handles the event by notifying the user and clearing caches.
    /// </summary>
    public async Task Handle(SubscriptionRenewedEvent @event, CancellationToken cancellationToken = default)
    {
        await _cacheService.RemoveByPrefixAsync($"subscription:{@event.SubscriptionId}", cancellationToken);

        await _notificationService.SendEmailAsync(
            @event.UserId,
            "Subscription Renewed",
            $"Your {@event.PlanName} subscription has been renewed successfully.",
            cancellationToken);
    }
}
