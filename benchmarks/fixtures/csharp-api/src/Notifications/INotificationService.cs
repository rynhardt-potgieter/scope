namespace CSharpApi.Notifications;

/// <summary>
/// Service for sending notifications to users via different channels.
/// </summary>
public interface INotificationService
{
    /// <summary>
    /// Send an email notification to the given address.
    /// Returns true if the email was sent successfully.
    /// </summary>
    bool SendEmail(string to, string subject, string body);

    /// <summary>
    /// Send an SMS notification to the given phone number.
    /// Returns true if the SMS was sent successfully.
    /// </summary>
    bool SendSms(string phoneNumber, string message);
}
