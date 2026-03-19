namespace CSharpLargeApi.Application.Commands.PublishContent;

/// <summary>
/// Command to publish a content item, making it publicly visible.
/// Requires the content to be in Draft or InReview status.
/// </summary>
public sealed class PublishContentCommand
{
    /// <summary>
    /// Gets the ID of the content item to publish.
    /// </summary>
    public Guid ContentId { get; }

    /// <summary>
    /// Gets the ID of the user requesting the publish action.
    /// Must have appropriate permissions.
    /// </summary>
    public Guid PublishedByUserId { get; }

    /// <summary>
    /// Creates a new PublishContentCommand.
    /// </summary>
    public PublishContentCommand(Guid contentId, Guid publishedByUserId)
    {
        ContentId = contentId;
        PublishedByUserId = publishedByUserId;
    }
}
