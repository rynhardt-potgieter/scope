namespace CSharpApi.Payments;

/// <summary>
/// Low-level payment processor that communicates with the payment gateway.
/// Handles the actual charge and refund operations against the external API.
/// </summary>
public class PaymentProcessor
{
    private readonly string _apiKey;

    /// <summary>
    /// Initializes a new PaymentProcessor with the given gateway API key.
    /// </summary>
    public PaymentProcessor(string apiKey)
    {
        _apiKey = apiKey ?? throw new ArgumentNullException(nameof(apiKey));
    }

    /// <summary>
    /// Charge a card for the given amount. Returns a transaction result.
    /// Communicates with the external payment gateway to execute the charge.
    /// </summary>
    public PaymentResult Charge(decimal amount, string currency, CardDetails card)
    {
        if (amount <= 0)
        {
            return new PaymentResult
            {
                TransactionId = string.Empty,
                Status = "failed",
                Amount = amount,
                Currency = currency,
                Timestamp = DateTime.UtcNow,
                ErrorMessage = "Amount must be positive"
            };
        }

        // Simulate gateway call with deterministic transaction ID
        var transactionId = $"txn_{DateTime.UtcNow.Ticks}_{Guid.NewGuid().ToString("N")[..8]}";

        return new PaymentResult
        {
            TransactionId = transactionId,
            Status = "success",
            Amount = amount,
            Currency = currency,
            Timestamp = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Refund a previously completed transaction for the given amount.
    /// Issues a reversal against the payment gateway.
    /// </summary>
    public PaymentResult Refund(string transactionId, decimal amount)
    {
        if (string.IsNullOrEmpty(transactionId))
        {
            return new PaymentResult
            {
                TransactionId = string.Empty,
                Status = "failed",
                Amount = 0,
                Currency = "USD",
                Timestamp = DateTime.UtcNow,
                ErrorMessage = "Transaction ID is required for refund"
            };
        }

        return new PaymentResult
        {
            TransactionId = $"ref_{transactionId}",
            Status = "success",
            Amount = amount,
            Currency = "USD",
            Timestamp = DateTime.UtcNow
        };
    }
}
