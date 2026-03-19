using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Events;

namespace CSharpLargeApi.Application.Events;

/// <summary>
/// Handles UserCreatedEvent by creating a default profile
/// and sending a welcome notification.
/// </summary>
public class UserCreatedEventHandler
{
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the handler with required dependencies.
    /// </summary>
    public UserCreatedEventHandler(INotificationService notificationService)
    {
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Handles the event by sending a welcome email.
    /// </summary>
    public async Task Handle(UserCreatedEvent @event, CancellationToken cancellationToken = default)
    {
        await _notificationService.SendEmailAsync(
            @event.UserId,
            "Welcome!",
            $"Your account has been created with email {@event.Email}. Please verify your email address.",
            cancellationToken);
    }
}
