using CSharpLargeApi.Infrastructure.External;

namespace CSharpLargeApi.Infrastructure.Services;

/// <summary>
/// Low-level email delivery service that handles email formatting,
/// template rendering, and delivery through SendGrid.
/// Used by the NotificationService for email channel delivery.
/// </summary>
public class EmailService
{
    private readonly SendGridClient _sendGridClient;
    private readonly string _fromAddress;
    private readonly string _fromName;

    /// <summary>
    /// Initializes the email service with the SendGrid client and sender details.
    /// </summary>
    public EmailService(SendGridClient sendGridClient, string fromAddress = "noreply@example.com", string fromName = "CSharpLargeApi")
    {
        _sendGridClient = sendGridClient ?? throw new ArgumentNullException(nameof(sendGridClient));
        _fromAddress = fromAddress;
        _fromName = fromName;
    }

    /// <summary>
    /// Sends a plain-text email to the specified recipient.
    /// </summary>
    public async Task SendPlainTextAsync(string toAddress, string subject, string body, CancellationToken cancellationToken = default)
    {
        var formattedBody = FormatPlainText(body);
        await _sendGridClient.SendEmailAsync(toAddress, subject, formattedBody, cancellationToken);
    }

    /// <summary>
    /// Sends an HTML email to the specified recipient.
    /// </summary>
    public async Task SendHtmlAsync(string toAddress, string subject, string htmlBody, CancellationToken cancellationToken = default)
    {
        var wrappedHtml = WrapInTemplate(subject, htmlBody);
        await _sendGridClient.SendEmailAsync(toAddress, subject, wrappedHtml, cancellationToken);
    }

    /// <summary>
    /// Sends a templated email using a named template and variable substitutions.
    /// </summary>
    public async Task SendTemplatedAsync(string toAddress, string templateName, Dictionary<string, string> variables, CancellationToken cancellationToken = default)
    {
        var subject = RenderTemplate($"{templateName}_subject", variables);
        var body = RenderTemplate($"{templateName}_body", variables);
        await SendHtmlAsync(toAddress, subject, body, cancellationToken);
    }

    private string FormatPlainText(string body)
    {
        return $"From: {_fromName} <{_fromAddress}>\n\n{body}\n\n---\nThis is an automated message.";
    }

    private string WrapInTemplate(string subject, string htmlBody)
    {
        return $"<html><head><title>{subject}</title></head><body><div class='container'>{htmlBody}</div><footer><p>Sent by {_fromName}</p></footer></body></html>";
    }

    private string RenderTemplate(string templateName, Dictionary<string, string> variables)
    {
        var template = $"[{templateName}]";
        foreach (var (key, value) in variables)
        {
            template = template.Replace($"{{{{{key}}}}}", value);
        }
        return template;
    }
}
