namespace CSharpLargeApi.Application.Commands.ProcessPayment;

/// <summary>
/// Command to process a payment for a user.
/// Contains all the information needed to charge a payment method
/// and record the resulting transaction.
/// </summary>
public sealed class ProcessPaymentCommand
{
    /// <summary>
    /// Gets the ID of the user making the payment.
    /// </summary>
    public Guid UserId { get; }

    /// <summary>
    /// Gets the payment amount in the smallest currency unit.
    /// </summary>
    public decimal Amount { get; }

    /// <summary>
    /// Gets the ISO 4217 currency code.
    /// </summary>
    public string Currency { get; }

    /// <summary>
    /// Gets the tokenized payment method identifier from the client.
    /// </summary>
    public string PaymentMethodToken { get; }

    /// <summary>
    /// Gets an optional idempotency key to prevent duplicate charges.
    /// </summary>
    public string? IdempotencyKey { get; }

    /// <summary>
    /// Gets an optional description for the payment.
    /// </summary>
    public string? Description { get; }

    /// <summary>
    /// Creates a new ProcessPaymentCommand.
    /// </summary>
    public ProcessPaymentCommand(
        Guid userId,
        decimal amount,
        string currency,
        string paymentMethodToken,
        string? idempotencyKey = null,
        string? description = null)
    {
        UserId = userId;
        Amount = amount;
        Currency = currency;
        PaymentMethodToken = paymentMethodToken;
        IdempotencyKey = idempotencyKey;
        Description = description;
    }
}
