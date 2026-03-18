namespace CSharpApi.Payments;

/// <summary>
/// Represents the outcome of a payment processing operation.
/// </summary>
public class PaymentResult
{
    /// <summary>Unique identifier for the transaction.</summary>
    public string TransactionId { get; set; } = string.Empty;

    /// <summary>Status of the transaction: "success", "failed", or "pending".</summary>
    public string Status { get; set; } = "pending";

    /// <summary>The charged or refunded amount.</summary>
    public decimal Amount { get; set; }

    /// <summary>Currency code (e.g. "USD").</summary>
    public string Currency { get; set; } = "USD";

    /// <summary>Timestamp of when the transaction was processed.</summary>
    public DateTime Timestamp { get; set; } = DateTime.UtcNow;

    /// <summary>Error message if the transaction failed, null otherwise.</summary>
    public string? ErrorMessage { get; set; }
}

/// <summary>
/// Represents a payment processing error with a reason code.
/// </summary>
public class PaymentError
{
    /// <summary>Machine-readable error code.</summary>
    public string Code { get; set; } = string.Empty;

    /// <summary>Human-readable error message.</summary>
    public string Message { get; set; } = string.Empty;

    /// <summary>The original transaction ID, if one was generated.</summary>
    public string? TransactionId { get; set; }
}
