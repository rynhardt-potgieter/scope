using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Exceptions;

namespace CSharpLargeApi.Application.Commands.PublishContent;

/// <summary>
/// Handles the PublishContentCommand by validating the content status,
/// publishing the content, and invalidating related caches.
/// </summary>
public class PublishContentHandler
{
    private readonly IContentService _contentService;
    private readonly ICacheService _cacheService;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the handler with required dependencies.
    /// </summary>
    public PublishContentHandler(
        IContentService contentService,
        ICacheService cacheService,
        INotificationService notificationService)
    {
        _contentService = contentService ?? throw new ArgumentNullException(nameof(contentService));
        _cacheService = cacheService ?? throw new ArgumentNullException(nameof(cacheService));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Handles the command by publishing the content and clearing caches.
    /// </summary>
    public async Task<ContentDto> Handle(PublishContentCommand command, CancellationToken cancellationToken)
    {
        var content = await _contentService.GetContentAsync(command.ContentId, cancellationToken);
        if (content is null)
        {
            throw new EntityNotFoundException("Content", command.ContentId);
        }

        if (content.Status != ContentStatus.Draft && content.Status != ContentStatus.InReview)
        {
            throw new BusinessRuleException(
                "PublishableStatusRequired",
                $"Content must be in Draft or InReview status to publish. Current status: {content.Status}",
                "Content");
        }

        var published = await _contentService.PublishContentAsync(command.ContentId, cancellationToken);

        await _cacheService.RemoveByPrefixAsync("content:", cancellationToken);

        await _notificationService.SendEmailAsync(
            content.AuthorId,
            "Content Published",
            $"Your content '{content.Title}' has been published successfully.",
            cancellationToken);

        return new ContentDto
        {
            Id = published.Id,
            Title = published.Title,
            Slug = published.Slug,
            AuthorId = published.AuthorId,
            Status = published.Status.ToString(),
            PublishedAt = published.PublishedAt,
            CreatedAt = published.CreatedAt
        };
    }
}
