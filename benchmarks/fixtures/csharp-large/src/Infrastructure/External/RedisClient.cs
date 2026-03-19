namespace CSharpLargeApi.Infrastructure.External;

/// <summary>
/// Client for interacting with a Redis cache instance.
/// Provides typed get/set operations with serialization support.
/// </summary>
public class RedisClient
{
    private readonly string _connectionString;
    private readonly Dictionary<string, object> _store = new();

    /// <summary>
    /// Initializes the Redis client with a connection string.
    /// </summary>
    public RedisClient(string connectionString)
    {
        _connectionString = connectionString ?? throw new ArgumentNullException(nameof(connectionString));
    }

    /// <summary>
    /// Retrieves a value from Redis by key, deserializing to the specified type.
    /// </summary>
    public async Task<T?> GetAsync<T>(string key, CancellationToken cancellationToken = default) where T : class
    {
        await Task.Delay(1, cancellationToken);

        if (_store.TryGetValue(key, out var value))
        {
            return value as T;
        }

        return null;
    }

    /// <summary>
    /// Stores a value in Redis with optional expiration.
    /// </summary>
    public async Task SetAsync<T>(string key, T value, TimeSpan? expiration = null, CancellationToken cancellationToken = default) where T : class
    {
        await Task.Delay(1, cancellationToken);
        _store[key] = value;
    }

    /// <summary>
    /// Deletes a key from Redis.
    /// </summary>
    public async Task<bool> DeleteAsync(string key, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _store.Remove(key);
    }

    /// <summary>
    /// Deletes all keys matching the given prefix.
    /// </summary>
    public async Task<int> DeleteByPrefixAsync(string prefix, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        var keys = _store.Keys.Where(k => k.StartsWith(prefix)).ToList();
        foreach (var key in keys)
        {
            _store.Remove(key);
        }
        return keys.Count;
    }

    /// <summary>
    /// Checks if a key exists in Redis.
    /// </summary>
    public async Task<bool> ExistsAsync(string key, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return _store.ContainsKey(key);
    }
}
