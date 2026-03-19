using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Exceptions;

namespace CSharpLargeApi.Application.Commands.ProcessRefund;

/// <summary>
/// Handles the ProcessRefundCommand by validating the original payment,
/// processing the refund through the payment service, and notifying the user.
/// Note: This handler calls RefundPayment, NOT ProcessPayment.
/// The RefundService.ProcessPartialRefund is the one that calls ProcessPayment.
/// </summary>
public class ProcessRefundHandler
{
    private readonly IPaymentService _paymentService;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the handler with required dependencies.
    /// </summary>
    public ProcessRefundHandler(
        IPaymentService paymentService,
        INotificationService notificationService)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Handles the refund command by validating and processing the refund.
    /// </summary>
    public async Task<PaymentDto> Handle(ProcessRefundCommand command, CancellationToken cancellationToken)
    {
        var payment = await _paymentService.GetPaymentAsync(command.PaymentId, cancellationToken);
        if (payment is null)
        {
            throw new EntityNotFoundException("Payment", command.PaymentId);
        }

        if (payment.Status != PaymentStatus.Completed)
        {
            throw new BusinessRuleException(
                "CompletedPaymentRequired",
                $"Only completed payments can be refunded. Current status: {payment.Status}",
                "Payment");
        }

        if (command.PartialAmount.HasValue && command.PartialAmount.Value > payment.Amount.Amount)
        {
            throw new BusinessRuleException(
                "RefundAmountExceedsPayment",
                $"Partial refund amount ({command.PartialAmount.Value}) exceeds original payment ({payment.Amount.Amount}).",
                "Payment");
        }

        var refundedPayment = await _paymentService.RefundPayment(
            command.PaymentId,
            command.Reason,
            cancellationToken);

        await _notificationService.SendEmailAsync(
            payment.UserId,
            "Refund Processed",
            $"Your refund of {payment.Amount} has been processed. Reason: {command.Reason}",
            cancellationToken);

        return new PaymentDto
        {
            Id = refundedPayment.Id,
            UserId = refundedPayment.UserId,
            Amount = refundedPayment.Amount.Amount,
            Currency = refundedPayment.Amount.Currency,
            Status = refundedPayment.Status.ToString(),
            GatewayTransactionId = refundedPayment.GatewayTransactionId,
            CreatedAt = refundedPayment.CreatedAt
        };
    }
}
