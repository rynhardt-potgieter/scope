using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Events;

namespace CSharpLargeApi.Application.Events;

/// <summary>
/// Handles PaymentProcessedEvent by updating related entities
/// and sending notifications. Runs after the payment is persisted.
/// </summary>
public class PaymentProcessedEventHandler
{
    private readonly INotificationService _notificationService;
    private readonly ICacheService _cacheService;

    /// <summary>
    /// Initializes the handler with required dependencies.
    /// </summary>
    public PaymentProcessedEventHandler(
        INotificationService notificationService,
        ICacheService cacheService)
    {
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
        _cacheService = cacheService ?? throw new ArgumentNullException(nameof(cacheService));
    }

    /// <summary>
    /// Handles the event by invalidating caches and sending notifications.
    /// </summary>
    public async Task Handle(PaymentProcessedEvent @event, CancellationToken cancellationToken = default)
    {
        // Invalidate user payment cache
        await _cacheService.RemoveByPrefixAsync($"payments:{@event.UserId}", cancellationToken);

        // Send receipt notification
        await _notificationService.SendEmailAsync(
            @event.UserId,
            "Payment Receipt",
            $"Payment of {@event.Amount} {@event.Currency} processed. ID: {@event.PaymentId}",
            cancellationToken);
    }
}
