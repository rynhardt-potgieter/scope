namespace CSharpLargeApi.Infrastructure.External;

/// <summary>
/// Client for interacting with the Stripe payment gateway.
/// Handles charge creation, refunds, and payment method validation.
/// </summary>
public class StripeClient
{
    private readonly string _apiKey;
    private readonly string _baseUrl;

    /// <summary>
    /// Initializes the Stripe client with API credentials.
    /// </summary>
    public StripeClient(string apiKey, string baseUrl = "https://api.stripe.com/v1")
    {
        _apiKey = apiKey ?? throw new ArgumentNullException(nameof(apiKey));
        _baseUrl = baseUrl;
    }

    /// <summary>
    /// Creates a charge against the specified payment method.
    /// Returns a gateway result with the transaction ID on success.
    /// </summary>
    public async Task<GatewayResult> ChargeAsync(decimal amount, string currency, string paymentMethodToken, CancellationToken cancellationToken = default)
    {
        // Simulate gateway call
        await Task.Delay(1, cancellationToken);

        if (string.IsNullOrWhiteSpace(paymentMethodToken))
        {
            return new GatewayResult
            {
                Succeeded = false,
                ErrorMessage = "Invalid payment method token."
            };
        }

        if (amount <= 0)
        {
            return new GatewayResult
            {
                Succeeded = false,
                ErrorMessage = "Charge amount must be positive."
            };
        }

        return new GatewayResult
        {
            Succeeded = true,
            TransactionId = $"ch_{Guid.NewGuid():N}",
            Amount = amount,
            Currency = currency
        };
    }

    /// <summary>
    /// Refunds a previously completed charge.
    /// </summary>
    public async Task<GatewayResult> RefundAsync(string transactionId, decimal amount, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);

        if (string.IsNullOrWhiteSpace(transactionId))
        {
            return new GatewayResult
            {
                Succeeded = false,
                ErrorMessage = "Transaction ID is required for refund."
            };
        }

        return new GatewayResult
        {
            Succeeded = true,
            TransactionId = $"re_{Guid.NewGuid():N}",
            Amount = amount
        };
    }

    /// <summary>
    /// Validates that a payment method token is still usable.
    /// </summary>
    public async Task<bool> ValidatePaymentMethodAsync(string paymentMethodToken, CancellationToken cancellationToken = default)
    {
        await Task.Delay(1, cancellationToken);
        return !string.IsNullOrWhiteSpace(paymentMethodToken) && paymentMethodToken.Length >= 8;
    }
}

/// <summary>
/// Represents the result of a payment gateway operation.
/// </summary>
public class GatewayResult
{
    /// <summary>
    /// Gets or sets whether the operation succeeded.
    /// </summary>
    public bool Succeeded { get; set; }

    /// <summary>
    /// Gets or sets the gateway transaction identifier.
    /// </summary>
    public string TransactionId { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the transaction amount.
    /// </summary>
    public decimal Amount { get; set; }

    /// <summary>
    /// Gets or sets the currency code.
    /// </summary>
    public string Currency { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the error message if the operation failed.
    /// </summary>
    public string? ErrorMessage { get; set; }
}
