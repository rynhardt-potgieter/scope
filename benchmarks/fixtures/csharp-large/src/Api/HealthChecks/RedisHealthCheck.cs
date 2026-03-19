using CSharpLargeApi.Infrastructure.External;

namespace CSharpLargeApi.Api.HealthChecks;

/// <summary>
/// Health check that verifies Redis cache connectivity.
/// </summary>
public class RedisHealthCheck
{
    private readonly RedisClient _redisClient;

    /// <summary>
    /// Initializes the health check with the Redis client.
    /// </summary>
    public RedisHealthCheck(RedisClient redisClient)
    {
        _redisClient = redisClient ?? throw new ArgumentNullException(nameof(redisClient));
    }

    /// <summary>
    /// Checks whether Redis is reachable by performing a ping operation.
    /// </summary>
    public async Task<HealthCheckResult> CheckAsync(CancellationToken cancellationToken = default)
    {
        try
        {
            var testKey = $"health:ping:{Guid.NewGuid():N}";
            await _redisClient.SetAsync(testKey, "pong", TimeSpan.FromSeconds(5), cancellationToken);
            var result = await _redisClient.GetAsync<string>(testKey, cancellationToken);
            await _redisClient.DeleteAsync(testKey, cancellationToken);

            if (result is not null)
            {
                return new HealthCheckResult(true, "Redis is healthy.");
            }

            return new HealthCheckResult(false, "Redis ping failed: no response.");
        }
        catch (Exception ex)
        {
            return new HealthCheckResult(false, $"Redis check failed: {ex.Message}");
        }
    }
}
