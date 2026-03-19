using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Application.Mappings;

/// <summary>
/// Provides mapping methods between Content entities and ContentDto objects.
/// Centralizes the mapping logic to avoid duplication across handlers.
/// </summary>
public static class ContentProfile
{
    /// <summary>
    /// Maps a Content entity to a ContentDto.
    /// Excludes the full body text for list views.
    /// </summary>
    public static ContentDto ToDto(Content content)
    {
        return new ContentDto
        {
            Id = content.Id,
            Title = content.Title,
            Slug = content.Slug,
            AuthorId = content.AuthorId,
            Status = content.Status.ToString(),
            CategoryId = content.CategoryId,
            PublishedAt = content.PublishedAt,
            CreatedAt = content.CreatedAt
        };
    }

    /// <summary>
    /// Maps a collection of Content entities to ContentDto objects.
    /// </summary>
    public static IReadOnlyList<ContentDto> ToDtoList(IEnumerable<Content> contentItems)
    {
        return contentItems.Select(ToDto).ToList().AsReadOnly();
    }
}
