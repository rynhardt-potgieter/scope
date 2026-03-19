using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Infrastructure.Workers;

/// <summary>
/// Background worker that retries failed payments on a scheduled basis.
/// Picks up failed payments, checks retry eligibility, and re-attempts
/// the charge through the payment service.
/// </summary>
public class PaymentRetryWorker
{
    private readonly IPaymentService _paymentService;
    private readonly IRepository<Payment> _paymentRepository;
    private readonly INotificationService _notificationService;
    private readonly int _maxRetries;

    /// <summary>
    /// Initializes the worker with required dependencies.
    /// </summary>
    public PaymentRetryWorker(
        IPaymentService paymentService,
        IRepository<Payment> paymentRepository,
        INotificationService notificationService,
        int maxRetries = 3)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _paymentRepository = paymentRepository ?? throw new ArgumentNullException(nameof(paymentRepository));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
        _maxRetries = maxRetries;
    }

    /// <summary>
    /// Executes the retry cycle, processing all eligible failed payments.
    /// </summary>
    public async Task ExecuteAsync(CancellationToken cancellationToken = default)
    {
        var allPayments = await _paymentRepository.GetAllAsync(cancellationToken);
        var failedPayments = allPayments
            .Where(p => p.Status == PaymentStatus.Failed && p.RetryCount < _maxRetries)
            .OrderBy(p => p.CreatedAt)
            .ToList();

        foreach (var payment in failedPayments)
        {
            if (cancellationToken.IsCancellationRequested)
                break;

            await RetryFailedPayment(payment, cancellationToken);
        }
    }

    /// <summary>
    /// Retries a single failed payment by creating a new charge attempt.
    /// Caller #5: This method calls IPaymentService.ProcessPayment to retry
    /// a previously failed payment transaction.
    /// </summary>
    public async Task RetryFailedPayment(Payment failedPayment, CancellationToken cancellationToken = default)
    {
        if (failedPayment.RetryCount >= _maxRetries)
        {
            await _notificationService.SendEmailAsync(
                failedPayment.UserId,
                "Payment Retry Limit Reached",
                $"Your payment of {failedPayment.Amount} has failed after {_maxRetries} attempts. Please update your payment method.",
                cancellationToken);
            return;
        }

        failedPayment.IncrementRetryCount();
        await _paymentRepository.SaveChangesAsync(cancellationToken);

        // Caller #5: PaymentRetryWorker.RetryFailedPayment calls ProcessPayment
        var retryResult = await _paymentService.ProcessPayment(
            failedPayment.UserId,
            failedPayment.Amount,
            $"retry_token_{failedPayment.Id}",
            cancellationToken);

        if (retryResult.Status == PaymentStatus.Completed)
        {
            await _notificationService.SendEmailAsync(
                failedPayment.UserId,
                "Payment Retry Successful",
                $"Your payment of {failedPayment.Amount} has been successfully processed on retry attempt {failedPayment.RetryCount}.",
                cancellationToken);
        }
        else
        {
            await _notificationService.SendEmailAsync(
                failedPayment.UserId,
                "Payment Retry Failed",
                $"Retry attempt {failedPayment.RetryCount}/{_maxRetries} for your payment of {failedPayment.Amount} has failed.",
                cancellationToken);
        }
    }
}
