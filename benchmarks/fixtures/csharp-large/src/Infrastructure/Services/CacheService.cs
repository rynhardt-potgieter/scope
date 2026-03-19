using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Infrastructure.External;

namespace CSharpLargeApi.Infrastructure.Services;

/// <summary>
/// Implementation of the caching service using Redis as the backing store.
/// Provides typed get/set operations with optional TTL expiration.
/// </summary>
public class CacheService : ICacheService
{
    private readonly RedisClient _redisClient;
    private readonly Dictionary<string, (object value, DateTime? expiry)> _localCache = new();

    /// <summary>
    /// Initializes the cache service with the Redis client.
    /// </summary>
    public CacheService(RedisClient redisClient)
    {
        _redisClient = redisClient ?? throw new ArgumentNullException(nameof(redisClient));
    }

    /// <summary>
    /// Retrieves a cached value by key.
    /// </summary>
    public async Task<T?> GetAsync<T>(string key, CancellationToken cancellationToken = default) where T : class
    {
        if (_localCache.TryGetValue(key, out var entry))
        {
            if (entry.expiry is null || entry.expiry > DateTime.UtcNow)
            {
                return entry.value as T;
            }
            _localCache.Remove(key);
        }

        var value = await _redisClient.GetAsync<T>(key, cancellationToken);
        if (value is not null)
        {
            _localCache[key] = (value, DateTime.UtcNow.AddMinutes(1));
        }

        return value;
    }

    /// <summary>
    /// Stores a value in the cache with optional expiration.
    /// </summary>
    public async Task SetAsync<T>(string key, T value, TimeSpan? expiration = null, CancellationToken cancellationToken = default) where T : class
    {
        var expiry = expiration.HasValue ? DateTime.UtcNow.Add(expiration.Value) : (DateTime?)null;
        _localCache[key] = (value, expiry);

        await _redisClient.SetAsync(key, value, expiration, cancellationToken);
    }

    /// <summary>
    /// Removes a cached value by key.
    /// </summary>
    public async Task RemoveAsync(string key, CancellationToken cancellationToken = default)
    {
        _localCache.Remove(key);
        await _redisClient.DeleteAsync(key, cancellationToken);
    }

    /// <summary>
    /// Removes all cached values matching the given prefix.
    /// </summary>
    public async Task RemoveByPrefixAsync(string prefix, CancellationToken cancellationToken = default)
    {
        var keysToRemove = _localCache.Keys.Where(k => k.StartsWith(prefix)).ToList();
        foreach (var key in keysToRemove)
        {
            _localCache.Remove(key);
        }

        await _redisClient.DeleteByPrefixAsync(prefix, cancellationToken);
    }

    /// <summary>
    /// Checks whether a key exists in the cache.
    /// </summary>
    public async Task<bool> ExistsAsync(string key, CancellationToken cancellationToken = default)
    {
        if (_localCache.ContainsKey(key))
        {
            return true;
        }

        return await _redisClient.ExistsAsync(key, cancellationToken);
    }
}
