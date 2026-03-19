namespace CSharpLargeApi.Shared.Constants;

/// <summary>
/// Standardized error codes used across the application.
/// These codes are included in error responses for programmatic handling.
/// </summary>
public static class ErrorCodes
{
    /// <summary>Error code for generic validation failures.</summary>
    public const string ValidationFailed = "VALIDATION_FAILED";

    /// <summary>Error code for entity not found errors.</summary>
    public const string NotFound = "NOT_FOUND";

    /// <summary>Error code for business rule violations.</summary>
    public const string BusinessRuleViolation = "BUSINESS_RULE_VIOLATION";

    /// <summary>Error code for authentication failures.</summary>
    public const string AuthenticationFailed = "AUTHENTICATION_FAILED";

    /// <summary>Error code for authorization failures.</summary>
    public const string AuthorizationFailed = "AUTHORIZATION_FAILED";

    /// <summary>Error code for rate limit exceeded.</summary>
    public const string RateLimitExceeded = "RATE_LIMIT_EXCEEDED";

    /// <summary>Error code for payment processing failures.</summary>
    public const string PaymentFailed = "PAYMENT_FAILED";

    /// <summary>Error code for refund processing failures.</summary>
    public const string RefundFailed = "REFUND_FAILED";

    /// <summary>Error code for duplicate entity errors.</summary>
    public const string DuplicateEntity = "DUPLICATE_ENTITY";

    /// <summary>Error code for internal server errors.</summary>
    public const string InternalError = "INTERNAL_ERROR";

    /// <summary>Error code for invalid payment method.</summary>
    public const string InvalidPaymentMethod = "INVALID_PAYMENT_METHOD";

    /// <summary>Error code for expired subscription.</summary>
    public const string SubscriptionExpired = "SUBSCRIPTION_EXPIRED";
}
