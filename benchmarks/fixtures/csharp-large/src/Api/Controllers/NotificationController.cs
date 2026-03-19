using CSharpLargeApi.Application.Interfaces;

namespace CSharpLargeApi.Api.Controllers;

/// <summary>
/// API controller for notification operations.
/// Provides endpoints for querying and managing user notifications.
/// </summary>
public class NotificationController
{
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the controller with the notification service.
    /// </summary>
    public NotificationController(INotificationService notificationService)
    {
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Gets the count of unread notifications for the specified user.
    /// GET /api/notifications/unread-count?userId={userId}
    /// </summary>
    public async Task<int> GetUnreadCount(Guid userId, CancellationToken cancellationToken = default)
    {
        return await _notificationService.GetUnreadCountAsync(userId, cancellationToken);
    }

    /// <summary>
    /// Marks all notifications as read for the specified user.
    /// POST /api/notifications/mark-all-read
    /// </summary>
    public async Task MarkAllAsRead(Guid userId, CancellationToken cancellationToken = default)
    {
        await _notificationService.MarkAllAsReadAsync(userId, cancellationToken);
    }

    /// <summary>
    /// Sends a test notification to the specified user.
    /// POST /api/notifications/test
    /// </summary>
    public async Task SendTestNotification(Guid userId, CancellationToken cancellationToken = default)
    {
        await _notificationService.SendEmailAsync(
            userId,
            "Test Notification",
            "This is a test notification to verify the notification system is working correctly.",
            cancellationToken);
    }
}
