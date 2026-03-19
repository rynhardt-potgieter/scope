namespace CSharpLargeApi.Shared.Configuration;

/// <summary>
/// Strongly-typed application settings.
/// Bound from configuration at startup.
/// </summary>
public class AppSettings
{
    /// <summary>Gets or sets the database connection settings.</summary>
    public DatabaseSettings Database { get; set; } = new();

    /// <summary>Gets or sets the Stripe payment gateway settings.</summary>
    public StripeSettings Stripe { get; set; } = new();

    /// <summary>Gets or sets the SendGrid email settings.</summary>
    public SendGridSettings SendGrid { get; set; } = new();

    /// <summary>Gets or sets the Redis cache settings.</summary>
    public RedisSettings Redis { get; set; } = new();

    /// <summary>Gets or sets the authentication settings.</summary>
    public AuthSettings Auth { get; set; } = new();

    /// <summary>Gets or sets the rate limiting settings.</summary>
    public RateLimitSettings RateLimit { get; set; } = new();
}

/// <summary>Database connection settings.</summary>
public class DatabaseSettings
{
    /// <summary>Gets or sets the connection string.</summary>
    public string ConnectionString { get; set; } = string.Empty;

    /// <summary>Gets or sets the command timeout in seconds.</summary>
    public int CommandTimeoutSeconds { get; set; } = 30;
}

/// <summary>Stripe payment gateway settings.</summary>
public class StripeSettings
{
    /// <summary>Gets or sets the API key.</summary>
    public string ApiKey { get; set; } = string.Empty;

    /// <summary>Gets or sets the webhook secret.</summary>
    public string WebhookSecret { get; set; } = string.Empty;
}

/// <summary>SendGrid email settings.</summary>
public class SendGridSettings
{
    /// <summary>Gets or sets the API key.</summary>
    public string ApiKey { get; set; } = string.Empty;

    /// <summary>Gets or sets the sender email address.</summary>
    public string FromAddress { get; set; } = "noreply@example.com";

    /// <summary>Gets or sets the sender display name.</summary>
    public string FromName { get; set; } = "CSharpLargeApi";
}

/// <summary>Redis cache settings.</summary>
public class RedisSettings
{
    /// <summary>Gets or sets the connection string.</summary>
    public string ConnectionString { get; set; } = "localhost:6379";

    /// <summary>Gets or sets the default cache expiration in minutes.</summary>
    public int DefaultExpirationMinutes { get; set; } = 5;
}

/// <summary>Authentication settings.</summary>
public class AuthSettings
{
    /// <summary>Gets or sets the JWT signing key.</summary>
    public string JwtSecret { get; set; } = string.Empty;

    /// <summary>Gets or sets the token expiration in minutes.</summary>
    public int TokenExpirationMinutes { get; set; } = 60;

    /// <summary>Gets or sets the refresh token expiration in days.</summary>
    public int RefreshTokenExpirationDays { get; set; } = 30;
}

/// <summary>Rate limiting settings.</summary>
public class RateLimitSettings
{
    /// <summary>Gets or sets the max requests per minute.</summary>
    public int RequestsPerMinute { get; set; } = 60;

    /// <summary>Gets or sets the max requests per hour.</summary>
    public int RequestsPerHour { get; set; } = 1000;
}
