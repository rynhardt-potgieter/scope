using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Mappings;
using CSharpLargeApi.Infrastructure.Services;

namespace CSharpLargeApi.Api.Controllers;

/// <summary>
/// API controller for refund operations.
/// Provides endpoints for processing full and partial refunds.
/// Note: This controller delegates to RefundService — it does NOT
/// call ProcessPayment directly. The RefundService.ProcessPartialRefund
/// is the one that calls ProcessPayment (caller #7).
/// </summary>
public class RefundController
{
    private readonly RefundService _refundService;

    /// <summary>
    /// Initializes the controller with the refund service.
    /// </summary>
    public RefundController(RefundService refundService)
    {
        _refundService = refundService ?? throw new ArgumentNullException(nameof(refundService));
    }

    /// <summary>
    /// Processes a full refund for a payment.
    /// POST /api/refunds
    /// </summary>
    public async Task<PaymentDto> ProcessRefund(Guid paymentId, string reason, CancellationToken cancellationToken = default)
    {
        var refund = await _refundService.ProcessRefund(paymentId, reason, cancellationToken);
        return PaymentProfile.ToDto(refund);
    }

    /// <summary>
    /// Processes a partial refund, creating a compensating charge for the remainder.
    /// POST /api/refunds/partial
    /// </summary>
    public async Task<PartialRefundResponse> ProcessPartialRefund(Guid paymentId, decimal refundAmount, string paymentMethodToken, string reason, CancellationToken cancellationToken = default)
    {
        var (refund, adjustment) = await _refundService.ProcessPartialRefund(
            paymentId, refundAmount, paymentMethodToken, reason, cancellationToken);

        return new PartialRefundResponse
        {
            Refund = PaymentProfile.ToDto(refund),
            Adjustment = PaymentProfile.ToDto(adjustment)
        };
    }
}

/// <summary>
/// Response object for partial refund operations.
/// Contains both the refund and the compensating adjustment payment.
/// </summary>
public class PartialRefundResponse
{
    /// <summary>
    /// Gets or sets the refunded payment details.
    /// </summary>
    public PaymentDto Refund { get; set; } = null!;

    /// <summary>
    /// Gets or sets the adjustment payment for the non-refunded portion.
    /// </summary>
    public PaymentDto Adjustment { get; set; } = null!;
}
