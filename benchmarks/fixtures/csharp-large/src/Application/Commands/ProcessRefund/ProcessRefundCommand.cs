namespace CSharpLargeApi.Application.Commands.ProcessRefund;

/// <summary>
/// Command to process a refund for a previously completed payment.
/// Supports both full and partial refunds.
/// </summary>
public sealed class ProcessRefundCommand
{
    /// <summary>
    /// Gets the ID of the payment to refund.
    /// </summary>
    public Guid PaymentId { get; }

    /// <summary>
    /// Gets the reason for the refund.
    /// </summary>
    public string Reason { get; }

    /// <summary>
    /// Gets the optional partial refund amount.
    /// If null, a full refund is processed.
    /// </summary>
    public decimal? PartialAmount { get; }

    /// <summary>
    /// Gets the ID of the user requesting the refund.
    /// </summary>
    public Guid RequestedByUserId { get; }

    /// <summary>
    /// Creates a new ProcessRefundCommand.
    /// </summary>
    public ProcessRefundCommand(Guid paymentId, string reason, Guid requestedByUserId, decimal? partialAmount = null)
    {
        PaymentId = paymentId;
        Reason = reason;
        RequestedByUserId = requestedByUserId;
        PartialAmount = partialAmount;
    }
}
