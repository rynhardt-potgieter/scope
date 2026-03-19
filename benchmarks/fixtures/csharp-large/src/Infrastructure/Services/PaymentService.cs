using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;
using CSharpLargeApi.Infrastructure.External;

namespace CSharpLargeApi.Infrastructure.Services;

/// <summary>
/// Implementation of the payment service that orchestrates payment processing.
/// This service contains THE ProcessPayment method that is the primary target
/// for caller analysis benchmarks. It coordinates between the payment gateway,
/// the repository, and the notification system.
/// </summary>
public class PaymentService : IPaymentService
{
    private readonly IRepository<Payment> _paymentRepository;
    private readonly StripeClient _stripeClient;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the payment service with all required dependencies.
    /// </summary>
    public PaymentService(
        IRepository<Payment> paymentRepository,
        StripeClient stripeClient,
        INotificationService notificationService)
    {
        _paymentRepository = paymentRepository ?? throw new ArgumentNullException(nameof(paymentRepository));
        _stripeClient = stripeClient ?? throw new ArgumentNullException(nameof(stripeClient));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Processes a payment for the specified user and amount.
    /// This is the primary entry point for all payment flows in the system.
    /// Validates the request, charges via the Stripe gateway, persists the
    /// transaction, and sends a confirmation notification.
    ///
    /// This method is called by exactly 7 non-test callers:
    /// 1. PaymentController.ProcessPayment
    /// 2. PaymentController.RetryPayment
    /// 3. SubscriptionController.RenewSubscription
    /// 4. ProcessPaymentHandler.Handle
    /// 5. PaymentRetryWorker.RetryFailedPayment
    /// 6. InvoiceService.SettleInvoice
    /// 7. RefundService.ProcessPartialRefund
    /// </summary>
    public async Task<Payment> ProcessPayment(Guid userId, Money amount, string paymentMethodToken, CancellationToken cancellationToken = default)
    {
        if (userId == Guid.Empty)
        {
            throw new BusinessRuleException("ValidUserRequired", "A valid user ID is required to process a payment.");
        }

        if (amount.Amount <= 0)
        {
            throw new BusinessRuleException("PositiveAmountRequired", "Payment amount must be greater than zero.");
        }

        if (string.IsNullOrWhiteSpace(paymentMethodToken))
        {
            throw new BusinessRuleException("PaymentMethodRequired", "A payment method token is required.");
        }

        var payment = Payment.Create(userId, amount);

        try
        {
            var gatewayResult = await _stripeClient.ChargeAsync(
                amount.Amount,
                amount.Currency,
                paymentMethodToken,
                cancellationToken);

            if (gatewayResult.Succeeded)
            {
                payment.MarkAsProcessed(gatewayResult.TransactionId);

                await _notificationService.SendEmailAsync(
                    userId,
                    "Payment Successful",
                    $"Your payment of {amount} has been processed. Transaction: {gatewayResult.TransactionId}",
                    cancellationToken);
            }
            else
            {
                payment.MarkAsFailed(gatewayResult.ErrorMessage ?? "Payment declined by gateway.");
            }
        }
        catch (Exception ex)
        {
            payment.MarkAsFailed($"Payment processing error: {ex.Message}");
        }

        await _paymentRepository.AddAsync(payment, cancellationToken);

        return payment;
    }

    /// <summary>
    /// Refunds a previously completed payment in full.
    /// </summary>
    public async Task<Payment> RefundPayment(Guid paymentId, string reason, CancellationToken cancellationToken = default)
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
                $"Only completed payments can be refunded. Current status: {payment.Status}");
        }

        if (string.IsNullOrWhiteSpace(payment.GatewayTransactionId))
        {
            throw new BusinessRuleException(
                "GatewayTransactionRequired",
                "Cannot refund a payment without a gateway transaction ID.");
        }

        var refundResult = await _stripeClient.RefundAsync(
            payment.GatewayTransactionId,
            payment.Amount.Amount,
            cancellationToken);

        if (refundResult.Succeeded)
        {
            payment.MarkAsRefunded();

            await _notificationService.SendEmailAsync(
                payment.UserId,
                "Refund Processed",
                $"Your refund of {payment.Amount} has been processed. Reason: {reason}",
                cancellationToken);
        }
        else
        {
            throw new BusinessRuleException(
                "RefundFailed",
                $"Refund failed: {refundResult.ErrorMessage}");
        }

        await _paymentRepository.SaveChangesAsync(cancellationToken);

        return payment;
    }

    /// <summary>
    /// Validates that a payment method token is still valid.
    /// </summary>
    public async Task<bool> ValidatePaymentMethod(string paymentMethodToken, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(paymentMethodToken))
        {
            return false;
        }

        return await _stripeClient.ValidatePaymentMethodAsync(paymentMethodToken, cancellationToken);
    }

    /// <summary>
    /// Retrieves a payment by its unique identifier.
    /// </summary>
    public async Task<Payment?> GetPaymentAsync(Guid paymentId, CancellationToken cancellationToken = default)
    {
        return await _paymentRepository.GetByIdAsync(paymentId, cancellationToken);
    }

    /// <summary>
    /// Lists all payments for a given user with pagination.
    /// </summary>
    public async Task<IReadOnlyList<Payment>> GetPaymentsByUserAsync(Guid userId, int skip = 0, int take = 20, CancellationToken cancellationToken = default)
    {
        var allPayments = await _paymentRepository.GetAllAsync(cancellationToken);
        return allPayments
            .Where(p => p.UserId == userId)
            .OrderByDescending(p => p.CreatedAt)
            .Skip(skip)
            .Take(take)
            .ToList()
            .AsReadOnly();
    }
}
