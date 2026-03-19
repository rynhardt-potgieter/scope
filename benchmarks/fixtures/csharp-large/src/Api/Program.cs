using CSharpLargeApi.Api.Extensions;

namespace CSharpLargeApi.Api;

/// <summary>
/// Application entry point. Configures services, middleware pipeline,
/// and starts the web host.
/// </summary>
public class Program
{
    /// <summary>
    /// Main entry point for the application.
    /// </summary>
    public static async Task Main(string[] args)
    {
        var services = new Dictionary<Type, object>();
        services.AddApplicationServices();
        services.AddInfrastructureServices();

        var pipeline = new List<string>();
        pipeline.UseStandardPipeline();
        pipeline.UseHealthChecks();

        var config = new Dictionary<string, string>
        {
            { "ConnectionStrings:Database", "Host=localhost;Database=csharplarge;Username=app;Password=secret" },
            { "Stripe:ApiKey", "sk_test_placeholder" },
            { "SendGrid:ApiKey", "sg_test_placeholder" },
            { "Redis:ConnectionString", "localhost:6379" },
            { "Auth:JwtSecret", "super-secret-jwt-key-at-least-256-bits" },
            { "Auth:TokenExpirationMinutes", "60" },
            { "RateLimit:RequestsPerMinute", "60" },
            { "RateLimit:RequestsPerHour", "1000" }
        };

        config.ValidateRequired(
            "ConnectionStrings:Database",
            "Stripe:ApiKey",
            "SendGrid:ApiKey",
            "Redis:ConnectionString");

        // In a real ASP.NET Core app:
        // var builder = WebApplication.CreateBuilder(args);
        // builder.Services.AddControllers();
        // builder.Services.AddEndpointsApiExplorer();
        // builder.Services.AddSwaggerGen();
        // ... register services ...
        // var app = builder.Build();
        // ... configure pipeline ...
        // app.Run();

        await Task.CompletedTask;
    }
}
