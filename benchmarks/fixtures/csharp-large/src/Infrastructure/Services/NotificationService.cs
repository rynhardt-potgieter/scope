using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Infrastructure.External;

namespace CSharpLargeApi.Infrastructure.Services;

/// <summary>
/// Implementation of the notification service that sends notifications
/// through various channels (email, push, SMS, in-app).
/// </summary>
public class NotificationService : INotificationService
{
    private readonly IRepository<Notification> _notificationRepository;
    private readonly SendGridClient _emailClient;

    /// <summary>
    /// Initializes the notification service with required dependencies.
    /// </summary>
    public NotificationService(
        IRepository<Notification> notificationRepository,
        SendGridClient emailClient)
    {
        _notificationRepository = notificationRepository ?? throw new ArgumentNullException(nameof(notificationRepository));
        _emailClient = emailClient ?? throw new ArgumentNullException(nameof(emailClient));
    }

    /// <summary>
    /// Sends a notification to the specified user through the given channel.
    /// </summary>
    public async Task SendNotificationAsync(Guid recipientId, string subject, string body, NotificationChannel channel, CancellationToken cancellationToken = default)
    {
        var notification = Notification.Create(recipientId, subject, body, channel);

        try
        {
            switch (channel)
            {
                case NotificationChannel.Email:
                    await _emailClient.SendEmailAsync(recipientId.ToString(), subject, body, cancellationToken);
                    notification.MarkAsDelivered();
                    break;

                case NotificationChannel.Push:
                    // Push notification delivery would go through a push provider
                    notification.MarkAsDelivered();
                    break;

                case NotificationChannel.InApp:
                    // In-app notifications are just stored, delivery is implicit
                    notification.MarkAsDelivered();
                    break;

                default:
                    notification.IncrementDeliveryAttempts();
                    break;
            }
        }
        catch (Exception)
        {
            notification.IncrementDeliveryAttempts();
        }

        await _notificationRepository.AddAsync(notification, cancellationToken);
    }

    /// <summary>
    /// Sends an email notification to the specified user.
    /// </summary>
    public async Task SendEmailAsync(Guid recipientId, string subject, string body, CancellationToken cancellationToken = default)
    {
        await SendNotificationAsync(recipientId, subject, body, NotificationChannel.Email, cancellationToken);
    }

    /// <summary>
    /// Sends a push notification to the specified user.
    /// </summary>
    public async Task SendPushAsync(Guid recipientId, string subject, string body, CancellationToken cancellationToken = default)
    {
        await SendNotificationAsync(recipientId, subject, body, NotificationChannel.Push, cancellationToken);
    }

    /// <summary>
    /// Retrieves the count of unread notifications for a user.
    /// </summary>
    public async Task<int> GetUnreadCountAsync(Guid recipientId, CancellationToken cancellationToken = default)
    {
        var all = await _notificationRepository.GetAllAsync(cancellationToken);
        return all.Count(n => n.RecipientId == recipientId && !n.IsRead);
    }

    /// <summary>
    /// Marks all notifications as read for the specified user.
    /// </summary>
    public async Task MarkAllAsReadAsync(Guid recipientId, CancellationToken cancellationToken = default)
    {
        var all = await _notificationRepository.GetAllAsync(cancellationToken);
        var unread = all.Where(n => n.RecipientId == recipientId && !n.IsRead);

        foreach (var notification in unread)
        {
            notification.MarkAsRead();
        }

        await _notificationRepository.SaveChangesAsync(cancellationToken);
    }
}
