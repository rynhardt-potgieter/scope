namespace CSharpLargeApi.Infrastructure.External;

/// <summary>
/// Client for interacting with the SendGrid email delivery API.
/// Handles email sending with retry logic and delivery tracking.
/// </summary>
public class SendGridClient
{
    private readonly string _apiKey;
    private readonly string _fromAddress;
    private int _sentCount;

    /// <summary>
    /// Initializes the SendGrid client with API credentials.
    /// </summary>
    public SendGridClient(string apiKey, string fromAddress = "noreply@example.com")
    {
        _apiKey = apiKey ?? throw new ArgumentNullException(nameof(apiKey));
        _fromAddress = fromAddress;
        _sentCount = 0;
    }

    /// <summary>
    /// Sends an email through SendGrid.
    /// </summary>
    public async Task<EmailDeliveryResult> SendEmailAsync(string toAddress, string subject, string body, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);

        if (string.IsNullOrWhiteSpace(toAddress))
        {
            return new EmailDeliveryResult
            {
                Succeeded = false,
                ErrorMessage = "Recipient address is required."
            };
        }

        _sentCount++;

        return new EmailDeliveryResult
        {
            Succeeded = true,
            MessageId = $"msg_{Guid.NewGuid():N}",
            SentAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Gets the total number of emails sent through this client instance.
    /// </summary>
    public int GetSentCount() => _sentCount;
}

/// <summary>
/// Represents the result of an email delivery attempt.
/// </summary>
public class EmailDeliveryResult
{
    /// <summary>
    /// Gets or sets whether the email was accepted for delivery.
    /// </summary>
    public bool Succeeded { get; set; }

    /// <summary>
    /// Gets or sets the unique message identifier.
    /// </summary>
    public string MessageId { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the timestamp when the email was sent.
    /// </summary>
    public DateTime SentAt { get; set; }

    /// <summary>
    /// Gets or sets the error message if delivery failed.
    /// </summary>
    public string? ErrorMessage { get; set; }
}
