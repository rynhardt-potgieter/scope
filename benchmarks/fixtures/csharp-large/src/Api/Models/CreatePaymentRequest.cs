namespace CSharpLargeApi.Api.Models;

/// <summary>
/// Request model for creating a new payment.
/// Binds to the JSON body of POST /api/payments.
/// </summary>
public class CreatePaymentRequest
{
    /// <summary>Gets or sets the user ID.</summary>
    public Guid UserId { get; set; }

    /// <summary>Gets or sets the payment amount.</summary>
    public decimal Amount { get; set; }

    /// <summary>Gets or sets the currency code.</summary>
    public string Currency { get; set; } = "USD";

    /// <summary>Gets or sets the tokenized payment method.</summary>
    public string PaymentMethodToken { get; set; } = string.Empty;

    /// <summary>Gets or sets the optional idempotency key.</summary>
    public string? IdempotencyKey { get; set; }

    /// <summary>Gets or sets the optional description.</summary>
    public string? Description { get; set; }
}
