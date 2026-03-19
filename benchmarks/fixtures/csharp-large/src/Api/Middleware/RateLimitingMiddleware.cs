using CSharpLargeApi.Application.Interfaces;

namespace CSharpLargeApi.Api.Middleware;

/// <summary>
/// Middleware that enforces rate limiting on API requests.
/// Uses a sliding window algorithm with configurable limits per IP or user.
/// </summary>
public class RateLimitingMiddleware
{
    private readonly Func<Task> _next;
    private readonly ICacheService _cacheService;
    private readonly int _maxRequestsPerMinute;
    private readonly int _maxRequestsPerHour;

    /// <summary>
    /// Initializes the middleware with rate limiting configuration.
    /// </summary>
    public RateLimitingMiddleware(Func<Task> next, ICacheService cacheService, int maxRequestsPerMinute = 60, int maxRequestsPerHour = 1000)
    {
        _next = next ?? throw new ArgumentNullException(nameof(next));
        _cacheService = cacheService ?? throw new ArgumentNullException(nameof(cacheService));
        _maxRequestsPerMinute = maxRequestsPerMinute;
        _maxRequestsPerHour = maxRequestsPerHour;
    }

    /// <summary>
    /// Invokes the middleware, checking and enforcing rate limits.
    /// </summary>
    public async Task InvokeAsync(string clientIdentifier)
    {
        var minuteKey = $"rate:{clientIdentifier}:min:{DateTime.UtcNow:yyyyMMddHHmm}";
        var hourKey = $"rate:{clientIdentifier}:hr:{DateTime.UtcNow:yyyyMMddHH}";

        var minuteCount = await GetRequestCountAsync(minuteKey);
        if (minuteCount >= _maxRequestsPerMinute)
        {
            throw new InvalidOperationException($"Rate limit exceeded: {_maxRequestsPerMinute} requests per minute.");
        }

        var hourCount = await GetRequestCountAsync(hourKey);
        if (hourCount >= _maxRequestsPerHour)
        {
            throw new InvalidOperationException($"Rate limit exceeded: {_maxRequestsPerHour} requests per hour.");
        }

        await IncrementRequestCountAsync(minuteKey, TimeSpan.FromMinutes(2));
        await IncrementRequestCountAsync(hourKey, TimeSpan.FromHours(2));

        await _next();
    }

    private async Task<int> GetRequestCountAsync(string key)
    {
        var cached = await _cacheService.GetAsync<RateCounter>(key);
        return cached?.Count ?? 0;
    }

    private async Task IncrementRequestCountAsync(string key, TimeSpan expiration)
    {
        var current = await _cacheService.GetAsync<RateCounter>(key);
        var counter = current ?? new RateCounter();
        counter.Count++;
        await _cacheService.SetAsync(key, counter, expiration);
    }
}

/// <summary>
/// Simple counter for rate limiting.
/// </summary>
public class RateCounter
{
    /// <summary>
    /// Gets or sets the request count.
    /// </summary>
    public int Count { get; set; }
}
