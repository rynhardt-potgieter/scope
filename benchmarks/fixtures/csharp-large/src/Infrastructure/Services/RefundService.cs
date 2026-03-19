using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Infrastructure.Services;

/// <summary>
/// Service responsible for processing refunds, including partial refunds
/// that require a new compensating payment. Handles the complex logic
/// of calculating refund amounts and creating adjustment transactions.
/// </summary>
public class RefundService
{
    private readonly IPaymentService _paymentService;
    private readonly IRepository<Payment> _paymentRepository;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the refund service with required dependencies.
    /// </summary>
    public RefundService(
        IPaymentService paymentService,
        IRepository<Payment> paymentRepository,
        INotificationService notificationService)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _paymentRepository = paymentRepository ?? throw new ArgumentNullException(nameof(paymentRepository));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Processes a full refund for a completed payment.
    /// Delegates to the payment service's RefundPayment method.
    /// </summary>
    public async Task<Payment> ProcessRefund(Guid paymentId, string reason, CancellationToken cancellationToken = default)
    {
        var payment = await _paymentRepository.GetByIdAsync(paymentId, cancellationToken);
        if (payment is null)
        {
            throw new EntityNotFoundException("Payment", paymentId);
        }

        if (payment.Status != PaymentStatus.Completed)
        {
            throw new BusinessRuleException(
                "CompletedPaymentRequired",
                "Only completed payments can be refunded.");
        }

        var refunded = await _paymentService.RefundPayment(paymentId, reason, cancellationToken);

        await _notificationService.SendEmailAsync(
            payment.UserId,
            "Full Refund Processed",
            $"A full refund of {payment.Amount} has been issued. Reason: {reason}",
            cancellationToken);

        return refunded;
    }

    /// <summary>
    /// Processes a partial refund by refunding the original payment in full
    /// and creating a new payment for the remaining amount.
    /// Caller #7: This method calls IPaymentService.ProcessPayment to create
    /// the compensating charge for the non-refunded portion.
    /// </summary>
    public async Task<(Payment refund, Payment adjustment)> ProcessPartialRefund(
        Guid paymentId,
        decimal refundAmount,
        string paymentMethodToken,
        string reason,
        CancellationToken cancellationToken = default)
    {
        var originalPayment = await _paymentRepository.GetByIdAsync(paymentId, cancellationToken);
        if (originalPayment is null)
        {
            throw new EntityNotFoundException("Payment", paymentId);
        }

        if (originalPayment.Status != PaymentStatus.Completed)
        {
            throw new BusinessRuleException(
                "CompletedPaymentRequired",
                "Only completed payments can be partially refunded.");
        }

        if (refundAmount <= 0 || refundAmount >= originalPayment.Amount.Amount)
        {
            throw new BusinessRuleException(
                "ValidPartialAmount",
                $"Partial refund amount must be between 0 and {originalPayment.Amount.Amount}.");
        }

        // Step 1: Refund the original payment in full
        var refundedPayment = await _paymentService.RefundPayment(paymentId, reason, cancellationToken);

        // Step 2: Create a new charge for the non-refunded portion
        var adjustmentAmount = originalPayment.Amount.Amount - refundAmount;
        var adjustmentMoney = new Money(adjustmentAmount, originalPayment.Amount.Currency);

        // Caller #7: RefundService.ProcessPartialRefund calls ProcessPayment
        var adjustmentPayment = await _paymentService.ProcessPayment(
            originalPayment.UserId,
            adjustmentMoney,
            paymentMethodToken,
            cancellationToken);

        await _notificationService.SendEmailAsync(
            originalPayment.UserId,
            "Partial Refund Processed",
            $"A partial refund of {new Money(refundAmount, originalPayment.Amount.Currency)} has been issued. " +
            $"A new charge of {adjustmentMoney} was created for the remaining balance. Reason: {reason}",
            cancellationToken);

        return (refundedPayment, adjustmentPayment);
    }
}
