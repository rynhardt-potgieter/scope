namespace CSharpApi.Payments;

/// <summary>
/// Represents details for a credit or debit card payment method.
/// </summary>
public record CardDetails(
    string CardNumber,
    int ExpiryMonth,
    int ExpiryYear,
    string Cvv,
    string CardholderName
);

/// <summary>
/// Request payload for processing a payment.
/// </summary>
public record PaymentRequest(
    string UserId,
    decimal Amount,
    string Currency,
    CardDetails Card,
    string? Description = null
);
