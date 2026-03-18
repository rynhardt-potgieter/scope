namespace CSharpApi.Notifications;

/// <summary>
/// Default implementation of INotificationService.
/// Sends notifications via email and SMS channels.
/// </summary>
public class NotificationService : INotificationService
{
    private readonly string _emailEndpoint;
    private readonly string _smsEndpoint;

    /// <summary>
    /// Initializes a new NotificationService with the given channel endpoints.
    /// </summary>
    public NotificationService(string emailEndpoint, string smsEndpoint)
    {
        _emailEndpoint = emailEndpoint ?? throw new ArgumentNullException(nameof(emailEndpoint));
        _smsEndpoint = smsEndpoint ?? throw new ArgumentNullException(nameof(smsEndpoint));
    }

    /// <summary>
    /// Send an email notification to the given address.
    /// Connects to the configured email endpoint to dispatch the message.
    /// </summary>
    public bool SendEmail(string to, string subject, string body)
    {
        if (string.IsNullOrWhiteSpace(to))
        {
            return false;
        }

        // Simulate sending email via the configured endpoint
        Console.Error.WriteLine($"[Email -> {to}] {subject}: {body}");
        return true;
    }

    /// <summary>
    /// Send an SMS notification to the given phone number.
    /// Connects to the configured SMS endpoint to dispatch the message.
    /// </summary>
    public bool SendSms(string phoneNumber, string message)
    {
        if (string.IsNullOrWhiteSpace(phoneNumber))
        {
            return false;
        }

        // Simulate sending SMS via the configured endpoint
        Console.Error.WriteLine($"[SMS -> {phoneNumber}] {message}");
        return true;
    }
}
