using CSharpLargeApi.Domain.Enums;

namespace CSharpLargeApi.Application.Interfaces;

/// <summary>
/// Defines the contract for sending notifications to users.
/// Supports multiple delivery channels and tracks delivery status.
/// </summary>
public interface INotificationService
{
    /// <summary>
    /// Sends a notification to the specified user through the given channel.
    /// </summary>
    Task SendNotificationAsync(Guid recipientId, string subject, string body, NotificationChannel channel, CancellationToken cancellationToken = default);

    /// <summary>
    /// Sends an email notification to the specified user.
    /// Convenience method that delegates to SendNotificationAsync with Email channel.
    /// </summary>
    Task SendEmailAsync(Guid recipientId, string subject, string body, CancellationToken cancellationToken = default);

    /// <summary>
    /// Sends a push notification to the specified user.
    /// </summary>
    Task SendPushAsync(Guid recipientId, string subject, string body, CancellationToken cancellationToken = default);

    /// <summary>
    /// Retrieves all unread notifications for the specified user.
    /// </summary>
    Task<int> GetUnreadCountAsync(Guid recipientId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Marks all notifications as read for the specified user.
    /// </summary>
    Task MarkAllAsReadAsync(Guid recipientId, CancellationToken cancellationToken = default);
}
