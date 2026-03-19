namespace CSharpLargeApi.Api.Models;

/// <summary>
/// Request model for processing a refund.
/// Binds to the JSON body of POST /api/refunds.
/// </summary>
public class RefundRequest
{
    /// <summary>Gets or sets the payment ID to refund.</summary>
    public Guid PaymentId { get; set; }

    /// <summary>Gets or sets the refund reason.</summary>
    public string Reason { get; set; } = string.Empty;

    /// <summary>Gets or sets the optional partial refund amount.</summary>
    public decimal? PartialAmount { get; set; }

    /// <summary>Gets or sets the payment method token for partial refund adjustment.</summary>
    public string? PaymentMethodToken { get; set; }
}
