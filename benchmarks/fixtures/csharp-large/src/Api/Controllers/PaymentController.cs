using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Application.Mappings;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Api.Controllers;

/// <summary>
/// API controller for payment-related operations.
/// Provides endpoints for processing payments, retrying failed payments,
/// and querying payment history.
/// </summary>
public class PaymentController
{
    private readonly IPaymentService _paymentService;

    /// <summary>
    /// Initializes the controller with the payment service.
    /// </summary>
    public PaymentController(IPaymentService paymentService)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
    }

    /// <summary>
    /// Processes a new payment for the specified user.
    /// Caller #1: PaymentController.ProcessPayment calls IPaymentService.ProcessPayment.
    /// POST /api/payments
    /// </summary>
    public async Task<PaymentDto> ProcessPayment(Guid userId, decimal amount, string currency, string paymentMethodToken, CancellationToken cancellationToken = default)
    {
        var money = new Money(amount, currency);

        // Caller #1: PaymentController.ProcessPayment calls ProcessPayment
        var payment = await _paymentService.ProcessPayment(
            userId, money, paymentMethodToken, cancellationToken);

        return PaymentProfile.ToDto(payment);
    }

    /// <summary>
    /// Retries a previously failed payment with a new or existing payment method.
    /// Caller #2: PaymentController.RetryPayment calls IPaymentService.ProcessPayment.
    /// POST /api/payments/retry
    /// </summary>
    public async Task<PaymentDto> RetryPayment(Guid userId, decimal amount, string currency, string paymentMethodToken, CancellationToken cancellationToken = default)
    {
        var money = new Money(amount, currency);

        // Caller #2: PaymentController.RetryPayment calls ProcessPayment
        var payment = await _paymentService.ProcessPayment(
            userId, money, paymentMethodToken, cancellationToken);

        return PaymentProfile.ToDto(payment);
    }

    /// <summary>
    /// Retrieves a specific payment by its identifier.
    /// GET /api/payments/{id}
    /// </summary>
    public async Task<PaymentDto?> GetPayment(Guid paymentId, CancellationToken cancellationToken = default)
    {
        var payment = await _paymentService.GetPaymentAsync(paymentId, cancellationToken);
        return payment is not null ? PaymentProfile.ToDto(payment) : null;
    }

    /// <summary>
    /// Lists payments for a specific user with pagination.
    /// GET /api/payments?userId={userId}&skip={skip}&take={take}
    /// </summary>
    public async Task<IReadOnlyList<PaymentDto>> ListPayments(Guid userId, int skip = 0, int take = 20, CancellationToken cancellationToken = default)
    {
        var payments = await _paymentService.GetPaymentsByUserAsync(
            userId, skip, take, cancellationToken);

        return PaymentProfile.ToDtoList(payments);
    }

    /// <summary>
    /// Validates a payment method token without creating a charge.
    /// POST /api/payments/validate-method
    /// </summary>
    public async Task<bool> ValidatePaymentMethod(string paymentMethodToken, CancellationToken cancellationToken = default)
    {
        return await _paymentService.ValidatePaymentMethod(paymentMethodToken, cancellationToken);
    }
}
