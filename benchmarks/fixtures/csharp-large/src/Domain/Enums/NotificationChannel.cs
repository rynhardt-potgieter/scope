namespace CSharpLargeApi.Domain.Enums;

/// <summary>
/// Defines the available channels for delivering notifications to users.
/// </summary>
public enum NotificationChannel
{
    /// <summary>Notification delivered via email.</summary>
    Email = 0,

    /// <summary>Notification delivered via SMS text message.</summary>
    Sms = 1,

    /// <summary>Notification delivered as a push notification.</summary>
    Push = 2,

    /// <summary>Notification stored in the in-app notification center.</summary>
    InApp = 3,

    /// <summary>Notification delivered via webhook to an external system.</summary>
    Webhook = 4
}
