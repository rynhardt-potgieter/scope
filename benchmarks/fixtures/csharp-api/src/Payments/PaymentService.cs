using CSharpApi.Notifications;

namespace CSharpApi.Payments;

/// <summary>
/// High-level payment service used by controllers and workers.
/// Orchestrates payment processing, validation, and notifications.
/// </summary>
public class PaymentService : IPaymentService
{
    private readonly PaymentProcessor _processor;
    private readonly INotificationService _notificationService;
    private readonly Dictionary<string, PaymentResult> _transactions = new();

    /// <summary>
    /// Initializes a new PaymentService with the required dependencies.
    /// </summary>
    public PaymentService(PaymentProcessor processor, INotificationService notificationService)
    {
        _processor = processor ?? throw new ArgumentNullException(nameof(processor));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Process a payment for the given amount and user.
    /// This is the primary entry point for all payment operations.
    /// Validates the card, charges via the processor, stores the transaction,
    /// and sends a confirmation notification.
    /// </summary>
    public PaymentResult ProcessPayment(decimal amount, string userId, CardDetails card)
    {
        ValidateCard(card);

        if (amount <= 0)
        {
            throw new ArgumentException("Payment amount must be positive", nameof(amount));
        }

        if (string.IsNullOrWhiteSpace(userId))
        {
            throw new ArgumentException("User ID is required", nameof(userId));
        }

        var result = _processor.Charge(amount, "USD", card);
        _transactions[result.TransactionId] = result;

        if (result.Status == "success")
        {
            _notificationService.SendEmail(
                userId,
                "Payment Confirmation",
                $"Your payment of {amount:C} was processed successfully. Transaction: {result.TransactionId}"
            );
        }

        return result;
    }

    /// <summary>
    /// Refund a previously completed payment by transaction ID.
    /// Looks up the original transaction and issues a reversal through the processor.
    /// </summary>
    public PaymentResult RefundPayment(string transactionId)
    {
        var original = GetTransaction(transactionId);
        if (original == null)
        {
            return new PaymentResult
            {
                TransactionId = string.Empty,
                Status = "failed",
                Amount = 0,
                Currency = "USD",
                Timestamp = DateTime.UtcNow,
                ErrorMessage = $"Transaction {transactionId} not found"
            };
        }

        var result = _processor.Refund(transactionId, original.Amount);
        _transactions[result.TransactionId] = result;

        if (result.Status == "success")
        {
            _notificationService.SendEmail(
                "refunds@system",
                "Refund Processed",
                $"Refund of {original.Amount:C} for transaction {transactionId} completed."
            );
        }

        return result;
    }

    /// <summary>
    /// Validate card details before processing.
    /// Throws ArgumentException if the card is invalid.
    /// </summary>
    public void ValidateCard(CardDetails card)
    {
        if (string.IsNullOrEmpty(card.CardNumber) || card.CardNumber.Length < 13)
        {
            throw new ArgumentException("Invalid card number");
        }

        if (string.IsNullOrEmpty(card.Cvv) || card.Cvv.Length < 3)
        {
            throw new ArgumentException("Invalid CVV");
        }

        if (card.ExpiryYear < DateTime.UtcNow.Year)
        {
            throw new ArgumentException("Card expired");
        }
    }

    /// <summary>
    /// Retrieve a transaction by its unique identifier.
    /// Returns null if no transaction is found with the given ID.
    /// </summary>
    public PaymentResult? GetTransaction(string transactionId)
    {
        _transactions.TryGetValue(transactionId, out var result);
        return result;
    }
}
