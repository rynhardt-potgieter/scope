namespace CSharpLargeApi.Application.DTOs;

/// <summary>
/// Data transfer object for payment information.
/// Used to return payment data from query and command handlers
/// without exposing domain entity internals.
/// </summary>
public class PaymentDto
{
    /// <summary>
    /// Gets or sets the payment identifier.
    /// </summary>
    public Guid Id { get; set; }

    /// <summary>
    /// Gets or sets the user identifier.
    /// </summary>
    public Guid UserId { get; set; }

    /// <summary>
    /// Gets or sets the payment amount.
    /// </summary>
    public decimal Amount { get; set; }

    /// <summary>
    /// Gets or sets the currency code.
    /// </summary>
    public string Currency { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the payment status.
    /// </summary>
    public string Status { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the external gateway transaction ID.
    /// </summary>
    public string? GatewayTransactionId { get; set; }

    /// <summary>
    /// Gets or sets the failure reason if payment failed.
    /// </summary>
    public string? FailureReason { get; set; }

    /// <summary>
    /// Gets or sets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; set; }
}
