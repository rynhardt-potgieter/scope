namespace CSharpLargeApi.Application.Interfaces;

/// <summary>
/// Defines the contract for a distributed caching service.
/// Implementations may use Redis, Memcached, or in-memory stores.
/// </summary>
public interface ICacheService
{
    /// <summary>
    /// Retrieves a cached value by key, deserializing it to the specified type.
    /// Returns default(T) if the key does not exist or has expired.
    /// </summary>
    Task<T?> GetAsync<T>(string key, CancellationToken cancellationToken = default) where T : class;

    /// <summary>
    /// Stores a value in the cache with the specified key and expiration.
    /// </summary>
    Task SetAsync<T>(string key, T value, TimeSpan? expiration = null, CancellationToken cancellationToken = default) where T : class;

    /// <summary>
    /// Removes a cached value by key.
    /// </summary>
    Task RemoveAsync(string key, CancellationToken cancellationToken = default);

    /// <summary>
    /// Removes all cached values matching the given key prefix.
    /// Useful for invalidating all cache entries for an entity type.
    /// </summary>
    Task RemoveByPrefixAsync(string prefix, CancellationToken cancellationToken = default);

    /// <summary>
    /// Checks whether a key exists in the cache.
    /// </summary>
    Task<bool> ExistsAsync(string key, CancellationToken cancellationToken = default);
}
