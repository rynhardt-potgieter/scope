namespace CSharpApi.Payments;

/// <summary>
/// High-level payment operations consumed by controllers and workers.
/// </summary>
public interface IPaymentService
{
    /// <summary>
    /// Process a payment for the given amount and user.
    /// This is the primary entry point for all payment operations.
    /// </summary>
    PaymentResult ProcessPayment(decimal amount, string userId, CardDetails card);

    /// <summary>
    /// Refund a previously completed payment by transaction ID.
    /// </summary>
    PaymentResult RefundPayment(string transactionId);

    /// <summary>
    /// Retrieve a transaction by its unique identifier.
    /// </summary>
    PaymentResult? GetTransaction(string transactionId);
}
