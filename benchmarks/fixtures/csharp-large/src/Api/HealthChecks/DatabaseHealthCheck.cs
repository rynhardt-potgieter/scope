using CSharpLargeApi.Infrastructure.Persistence;

namespace CSharpLargeApi.Api.HealthChecks;

/// <summary>
/// Health check that verifies database connectivity.
/// Returns unhealthy if the database cannot be reached.
/// </summary>
public class DatabaseHealthCheck
{
    private readonly AppDbContext _context;

    /// <summary>
    /// Initializes the health check with the database context.
    /// </summary>
    public DatabaseHealthCheck(AppDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <summary>
    /// Checks whether the database is reachable and responsive.
    /// </summary>
    public async Task<HealthCheckResult> CheckAsync(CancellationToken cancellationToken = default)
    {
        try
        {
            await _context.SaveChangesAsync(cancellationToken);
            return new HealthCheckResult(true, "Database is healthy.");
        }
        catch (Exception ex)
        {
            return new HealthCheckResult(false, $"Database check failed: {ex.Message}");
        }
    }
}

/// <summary>
/// Represents the result of a health check.
/// </summary>
public class HealthCheckResult
{
    /// <summary>Gets whether the component is healthy.</summary>
    public bool IsHealthy { get; }

    /// <summary>Gets a description of the health status.</summary>
    public string Description { get; }

    /// <summary>Creates a new health check result.</summary>
    public HealthCheckResult(bool isHealthy, string description)
    {
        IsHealthy = isHealthy;
        Description = description;
    }
}
