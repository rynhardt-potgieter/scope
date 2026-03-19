namespace CSharpLargeApi.Domain.Enums;

/// <summary>
/// Represents the publication lifecycle of a content item.
/// </summary>
public enum ContentStatus
{
    /// <summary>Content is in draft and not yet submitted for review.</summary>
    Draft = 0,

    /// <summary>Content has been submitted and is awaiting editorial review.</summary>
    InReview = 1,

    /// <summary>Content has been approved and is publicly visible.</summary>
    Published = 2,

    /// <summary>Content has been unpublished and is no longer visible.</summary>
    Archived = 3,

    /// <summary>Content was rejected during review.</summary>
    Rejected = 4
}
