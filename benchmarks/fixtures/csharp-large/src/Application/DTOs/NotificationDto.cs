namespace CSharpLargeApi.Application.DTOs;

/// <summary>
/// Data transfer object for notification information.
/// </summary>
public class NotificationDto
{
    /// <summary>
    /// Gets or sets the notification identifier.
    /// </summary>
    public Guid Id { get; set; }

    /// <summary>
    /// Gets or sets the recipient user identifier.
    /// </summary>
    public Guid RecipientId { get; set; }

    /// <summary>
    /// Gets or sets the notification subject.
    /// </summary>
    public string Subject { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the notification body.
    /// </summary>
    public string Body { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the delivery channel name.
    /// </summary>
    public string Channel { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets whether the notification was delivered.
    /// </summary>
    public bool IsDelivered { get; set; }

    /// <summary>
    /// Gets or sets whether the notification was read.
    /// </summary>
    public bool IsRead { get; set; }

    /// <summary>
    /// Gets or sets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; set; }
}
