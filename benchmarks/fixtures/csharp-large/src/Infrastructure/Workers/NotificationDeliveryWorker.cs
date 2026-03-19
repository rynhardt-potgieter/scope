using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Infrastructure.Workers;

/// <summary>
/// Background worker that processes queued notifications and retries
/// failed deliveries. Runs on a schedule to pick up undelivered
/// notifications and attempt redelivery.
/// </summary>
public class NotificationDeliveryWorker
{
    private readonly IRepository<Notification> _notificationRepository;
    private readonly INotificationService _notificationService;
    private readonly int _maxDeliveryAttempts;

    /// <summary>
    /// Initializes the worker with required dependencies.
    /// </summary>
    public NotificationDeliveryWorker(
        IRepository<Notification> notificationRepository,
        INotificationService notificationService,
        int maxDeliveryAttempts = 5)
    {
        _notificationRepository = notificationRepository ?? throw new ArgumentNullException(nameof(notificationRepository));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
        _maxDeliveryAttempts = maxDeliveryAttempts;
    }

    /// <summary>
    /// Executes the delivery retry cycle for all undelivered notifications.
    /// </summary>
    public async Task ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var allNotifications = await _notificationRepository.GetAllAsync(cancellationToken);
        var undelivered = allNotifications
            .Where(n => !n.IsDelivered && n.DeliveryAttempts < _maxDeliveryAttempts)
            .OrderBy(n => n.CreatedAt)
            .ToList();

        foreach (var notification in undelivered)
        {
            if (cancellationToken.IsCancellationRequested)
                break;

            await RetryDelivery(notification, cancellationToken);
        }
    }

    /// <summary>
    /// Retries delivery for a single undelivered notification.
    /// </summary>
    private async Task RetryDelivery(Notification notification, CancellationToken cancellationToken)
    {
        try
        {
            await _notificationService.SendNotificationAsync(
                notification.RecipientId,
                notification.Subject,
                notification.Body,
                notification.Channel,
                cancellationToken);
        }
        catch (Exception)
        {
            notification.IncrementDeliveryAttempts();
            await _notificationRepository.SaveChangesAsync(cancellationToken);
        }
    }
}
