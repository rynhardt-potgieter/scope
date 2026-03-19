namespace CSharpLargeApi.Shared.Constants;

/// <summary>
/// Application-wide constants used across all layers.
/// </summary>
public static class AppConstants
{
    /// <summary>
    /// The default page size for paginated queries.
    /// </summary>
    public const int DefaultPageSize = 20;

    /// <summary>
    /// The maximum allowed page size for paginated queries.
    /// </summary>
    public const int MaxPageSize = 100;

    /// <summary>
    /// The default currency code used when none is specified.
    /// </summary>
    public const string DefaultCurrency = "USD";

    /// <summary>
    /// The maximum number of payment retry attempts.
    /// </summary>
    public const int MaxPaymentRetries = 3;

    /// <summary>
    /// The maximum number of notification delivery attempts.
    /// </summary>
    public const int MaxNotificationDeliveryAttempts = 5;

    /// <summary>
    /// The default session duration in days.
    /// </summary>
    public const int DefaultSessionDurationDays = 30;

    /// <summary>
    /// The default access token expiration in minutes.
    /// </summary>
    public const int DefaultAccessTokenExpirationMinutes = 60;

    /// <summary>
    /// The application name used in email headers and logs.
    /// </summary>
    public const string ApplicationName = "CSharpLargeApi";

    /// <summary>
    /// The minimum password length required for user accounts.
    /// </summary>
    public const int MinPasswordLength = 8;

    /// <summary>
    /// The maximum display name length.
    /// </summary>
    public const int MaxDisplayNameLength = 100;
}
