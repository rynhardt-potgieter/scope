namespace CSharpLargeApi.Api.Extensions;

/// <summary>
/// Extension methods for configuring the middleware pipeline.
/// Provides a fluent API for adding middleware in the correct order.
/// </summary>
public static class MiddlewareExtensions
{
    /// <summary>
    /// Configures the standard middleware pipeline for the API.
    /// Order: Exception Handler -> Authentication -> Rate Limiting -> Audit -> Endpoint.
    /// </summary>
    public static List<string> UseStandardPipeline(this List<string> pipeline)
    {
        pipeline.Add("ExceptionHandlerMiddleware");
        pipeline.Add("AuthenticationMiddleware");
        pipeline.Add("RateLimitingMiddleware");
        pipeline.Add("AuditMiddleware");
        pipeline.Add("EndpointRouting");
        return pipeline;
    }

    /// <summary>
    /// Adds development-only middleware (detailed errors, Swagger, etc.).
    /// </summary>
    public static List<string> UseDevelopmentMiddleware(this List<string> pipeline)
    {
        pipeline.Insert(0, "DeveloperExceptionPage");
        pipeline.Add("SwaggerUI");
        return pipeline;
    }

    /// <summary>
    /// Adds health check endpoints to the pipeline.
    /// </summary>
    public static List<string> UseHealthChecks(this List<string> pipeline)
    {
        pipeline.Add("HealthChecks:/health");
        pipeline.Add("HealthChecks:/health/ready");
        pipeline.Add("HealthChecks:/health/live");
        return pipeline;
    }
}
