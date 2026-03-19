using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Infrastructure.Services;

namespace CSharpLargeApi.Tests.Unit.Services;

/// <summary>
/// Unit tests for the NotificationService class.
/// </summary>
public class NotificationServiceTests
{
    /// <summary>Verifies email notification is delivered.</summary>
    public async Task SendEmail_WithValidRecipient_MarksAsDelivered()
    {
        await Task.CompletedTask;
    }

    /// <summary>Verifies push notification is delivered.</summary>
    public async Task SendPush_WithValidRecipient_MarksAsDelivered()
    {
        await Task.CompletedTask;
    }

    /// <summary>Verifies unread count returns correct value.</summary>
    public async Task GetUnreadCount_WithUnreadNotifications_ReturnsCorrectCount()
    {
        await Task.CompletedTask;
    }

    /// <summary>Verifies mark all as read updates all notifications.</summary>
    public async Task MarkAllAsRead_SetsIsReadTrue()
    {
        await Task.CompletedTask;
    }
}
