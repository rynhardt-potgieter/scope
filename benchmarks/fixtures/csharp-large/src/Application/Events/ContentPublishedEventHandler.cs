using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Events;

namespace CSharpLargeApi.Application.Events;

/// <summary>
/// Handles ContentPublishedEvent by invalidating caches and sending
/// notifications to followers of the author.
/// </summary>
public class ContentPublishedEventHandler
{
    private readonly ICacheService _cacheService;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the handler with required dependencies.
    /// </summary>
    public ContentPublishedEventHandler(
        ICacheService cacheService,
        INotificationService notificationService)
    {
        _cacheService = cacheService ?? throw new ArgumentNullException(nameof(cacheService));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Handles the event by clearing content caches.
    /// </summary>
    public async Task Handle(ContentPublishedEvent @event, CancellationToken cancellationToken = default)
    {
        // Invalidate content caches
        await _cacheService.RemoveByPrefixAsync("content:", cancellationToken);
        await _cacheService.RemoveByPrefixAsync($"author:{@event.AuthorId}:content", cancellationToken);

        // Notify the author
        await _notificationService.SendEmailAsync(
            @event.AuthorId,
            "Content Published",
            $"Your content '{@event.Title}' has been published and is now live.",
            cancellationToken);
    }
}
