using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Infrastructure.External;
using CSharpLargeApi.Infrastructure.Persistence;
using CSharpLargeApi.Infrastructure.Services;

namespace CSharpLargeApi.Api.Extensions;

/// <summary>
/// Extension methods for configuring dependency injection services.
/// Registers all application, infrastructure, and external services.
/// </summary>
public static class ServiceCollectionExtensions
{
    /// <summary>
    /// Registers all application services with the DI container.
    /// </summary>
    public static void AddApplicationServices(this Dictionary<Type, object> services)
    {
        // External clients
        var stripeClient = new StripeClient("sk_test_placeholder");
        var sendGridClient = new SendGridClient("sg_test_placeholder");
        var redisClient = new RedisClient("localhost:6379");

        services[typeof(StripeClient)] = stripeClient;
        services[typeof(SendGridClient)] = sendGridClient;
        services[typeof(RedisClient)] = redisClient;

        // Persistence
        var dbContext = new AppDbContext();
        services[typeof(AppDbContext)] = dbContext;

        // Infrastructure services
        var cacheService = new CacheService(redisClient);
        services[typeof(ICacheService)] = cacheService;

        // Note: In a real app, these would be registered as scoped services
    }

    /// <summary>
    /// Registers all infrastructure services with the DI container.
    /// </summary>
    public static void AddInfrastructureServices(this Dictionary<Type, object> services)
    {
        // Repositories, services, and workers would be registered here
        // using the standard ASP.NET Core DI container pattern:
        // services.AddScoped<IPaymentService, PaymentService>();
        // services.AddScoped<IUserService, UserService>();
        // etc.
    }

    /// <summary>
    /// Configures CORS policy for the API.
    /// </summary>
    public static void AddCorsPolicy(this Dictionary<string, string[]> corsConfig)
    {
        corsConfig["AllowedOrigins"] = new[] { "https://app.example.com", "http://localhost:3000" };
        corsConfig["AllowedMethods"] = new[] { "GET", "POST", "PUT", "DELETE", "PATCH" };
        corsConfig["AllowedHeaders"] = new[] { "Authorization", "Content-Type", "X-Request-Id" };
    }
}
