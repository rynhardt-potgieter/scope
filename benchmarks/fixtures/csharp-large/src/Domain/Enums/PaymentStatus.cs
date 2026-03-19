namespace CSharpLargeApi.Domain.Enums;

/// <summary>
/// Represents the lifecycle status of a payment transaction.
/// </summary>
public enum PaymentStatus
{
    /// <summary>Payment has been created but not yet processed.</summary>
    Pending = 0,

    /// <summary>Payment is currently being processed by the gateway.</summary>
    Processing = 1,

    /// <summary>Payment was successfully completed.</summary>
    Completed = 2,

    /// <summary>Payment failed during processing.</summary>
    Failed = 3,

    /// <summary>Payment was cancelled before processing.</summary>
    Cancelled = 4,

    /// <summary>Payment was refunded after completion.</summary>
    Refunded = 5,

    /// <summary>Payment was partially refunded.</summary>
    PartiallyRefunded = 6
}
