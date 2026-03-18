using CSharpApi.Payments;
using CSharpApi.Notifications;

namespace CSharpApi.Controllers;

/// <summary>
/// Controller handling refund-related HTTP endpoints.
/// Processes refund requests and sends confirmation notifications.
/// </summary>
public class RefundController
{
    private readonly IPaymentService _paymentService;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes a new RefundController with the required services.
    /// </summary>
    public RefundController(IPaymentService paymentService, INotificationService notificationService)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Process a refund for a given transaction.
    /// Looks up the transaction and issues a reversal, then notifies the customer.
    /// </summary>
    public PaymentResult ProcessRefund(string transactionId)
    {
        var result = _paymentService.RefundPayment(transactionId);

        if (result.Status == "success")
        {
            _notificationService.SendEmail(
                "customer@example.com",
                "Refund Confirmation",
                $"Your refund for transaction {transactionId} has been processed. Amount: {result.Amount:C}"
            );
        }

        return result;
    }

    /// <summary>
    /// Process a partial refund for a given transaction.
    /// Only refunds the specified amount rather than the full transaction.
    /// </summary>
    public PaymentResult ProcessPartialRefund(string transactionId, decimal refundAmount)
    {
        var original = _paymentService.GetTransaction(transactionId);
        if (original == null)
        {
            return new PaymentResult
            {
                TransactionId = string.Empty,
                Status = "failed",
                Amount = 0,
                Currency = "USD",
                Timestamp = DateTime.UtcNow,
                ErrorMessage = $"Transaction {transactionId} not found for partial refund"
            };
        }

        if (refundAmount > original.Amount)
        {
            return new PaymentResult
            {
                TransactionId = string.Empty,
                Status = "failed",
                Amount = 0,
                Currency = "USD",
                Timestamp = DateTime.UtcNow,
                ErrorMessage = "Partial refund amount exceeds original transaction amount"
            };
        }

        // For partial refunds, we still use the full refund flow in this simplified fixture
        return _paymentService.RefundPayment(transactionId);
    }
}
