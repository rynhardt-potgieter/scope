using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Application.Mappings;
using CSharpLargeApi.Domain.Enums;

namespace CSharpLargeApi.Application.Services;

/// <summary>
/// Application service that provides advanced content search capabilities.
/// Wraps the content service with caching and result enrichment.
/// </summary>
public class ContentSearchService
{
    private readonly IContentService _contentService;
    private readonly ICacheService _cacheService;

    /// <summary>
    /// Initializes the service with required dependencies.
    /// </summary>
    public ContentSearchService(IContentService contentService, ICacheService cacheService)
    {
        _contentService = contentService ?? throw new ArgumentNullException(nameof(contentService));
        _cacheService = cacheService ?? throw new ArgumentNullException(nameof(cacheService));
    }

    /// <summary>
    /// Searches for published content with caching.
    /// </summary>
    public async Task<IReadOnlyList<ContentDto>> SearchPublishedAsync(string query, int skip = 0, int take = 20, CancellationToken cancellationToken = default)
    {
        var cacheKey = $"search:published:{query}:{skip}:{take}";
        var cached = await _cacheService.GetAsync<List<ContentDto>>(cacheKey, cancellationToken);
        if (cached is not null)
        {
            return cached.AsReadOnly();
        }

        var results = await _contentService.SearchContentAsync(
            query, ContentStatus.Published, skip, take, cancellationToken);

        var dtos = ContentProfile.ToDtoList(results);

        await _cacheService.SetAsync(cacheKey, dtos.ToList(), TimeSpan.FromMinutes(5), cancellationToken);

        return dtos;
    }

    /// <summary>
    /// Gets trending content based on recent publication dates.
    /// </summary>
    public async Task<IReadOnlyList<ContentDto>> GetTrendingAsync(int count = 10, CancellationToken cancellationToken = default)
    {
        var cacheKey = $"content:trending:{count}";
        var cached = await _cacheService.GetAsync<List<ContentDto>>(cacheKey, cancellationToken);
        if (cached is not null)
        {
            return cached.AsReadOnly();
        }

        var results = await _contentService.SearchContentAsync(
            "", ContentStatus.Published, 0, count, cancellationToken);

        var dtos = ContentProfile.ToDtoList(results);

        await _cacheService.SetAsync(cacheKey, dtos.ToList(), TimeSpan.FromMinutes(15), cancellationToken);

        return dtos;
    }
}
