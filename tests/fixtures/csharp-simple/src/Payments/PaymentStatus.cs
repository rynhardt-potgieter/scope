namespace Payments;

/// <summary>
/// Status of a payment transaction.
/// </summary>
public enum PaymentStatus
{
    Pending,
    Completed,
    Failed,
    Refunded = 10,
}
