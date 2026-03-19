using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.ValueObjects;
using CSharpLargeApi.Infrastructure.Services;

namespace CSharpLargeApi.Api.Controllers;

/// <summary>
/// API controller for subscription management operations.
/// Provides endpoints for creating, renewing, and cancelling subscriptions.
/// </summary>
public class SubscriptionController
{
    private readonly SubscriptionService _subscriptionService;
    private readonly IPaymentService _paymentService;

    /// <summary>
    /// Initializes the controller with the subscription and payment services.
    /// </summary>
    public SubscriptionController(SubscriptionService subscriptionService, IPaymentService paymentService)
    {
        _subscriptionService = subscriptionService ?? throw new ArgumentNullException(nameof(subscriptionService));
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
    }

    /// <summary>
    /// Creates a new subscription for a user.
    /// POST /api/subscriptions
    /// </summary>
    public async Task<SubscriptionDto> Create(Guid userId, string planName, decimal monthlyPrice, string currency, string paymentMethodToken, CancellationToken cancellationToken = default)
    {
        var price = new Money(monthlyPrice, currency);
        var subscription = await _subscriptionService.CreateSubscription(userId, planName, price, cancellationToken);

        return new SubscriptionDto
        {
            Id = subscription.Id,
            UserId = subscription.UserId,
            PlanName = subscription.PlanName,
            MonthlyPrice = subscription.Price.Amount,
            Currency = subscription.Price.Currency,
            Status = subscription.Status.ToString(),
            NextRenewalDate = subscription.NextRenewalDate,
            CreatedAt = subscription.CreatedAt
        };
    }

    /// <summary>
    /// Renews a subscription and processes the renewal payment.
    /// Caller #3: SubscriptionController.RenewSubscription calls IPaymentService.ProcessPayment.
    /// POST /api/subscriptions/{id}/renew
    /// </summary>
    public async Task<SubscriptionDto> RenewSubscription(Guid subscriptionId, string paymentMethodToken, CancellationToken cancellationToken = default)
    {
        var subscription = await _subscriptionService.GetSubscriptionAsync(subscriptionId, cancellationToken);
        if (subscription is null)
        {
            throw new EntityNotFoundException("Subscription", subscriptionId);
        }

        // Process the renewal payment first
        // Caller #3: SubscriptionController.RenewSubscription calls ProcessPayment
        var payment = await _paymentService.ProcessPayment(
            subscription.UserId,
            subscription.Price,
            paymentMethodToken,
            cancellationToken);

        if (payment.Status != Domain.Enums.PaymentStatus.Completed)
        {
            throw new BusinessRuleException(
                "RenewalPaymentFailed",
                $"Renewal payment failed: {payment.FailureReason}");
        }

        // Then renew the subscription period
        var renewed = await _subscriptionService.RenewSubscription(subscriptionId, cancellationToken);

        return new SubscriptionDto
        {
            Id = renewed.Id,
            UserId = renewed.UserId,
            PlanName = renewed.PlanName,
            MonthlyPrice = renewed.Price.Amount,
            Currency = renewed.Price.Currency,
            Status = renewed.Status.ToString(),
            NextRenewalDate = renewed.NextRenewalDate,
            CreatedAt = renewed.CreatedAt
        };
    }

    /// <summary>
    /// Cancels a subscription.
    /// POST /api/subscriptions/{id}/cancel
    /// </summary>
    public async Task<SubscriptionDto> Cancel(Guid subscriptionId, CancellationToken cancellationToken = default)
    {
        var cancelled = await _subscriptionService.CancelSubscription(subscriptionId, cancellationToken);

        return new SubscriptionDto
        {
            Id = cancelled.Id,
            UserId = cancelled.UserId,
            PlanName = cancelled.PlanName,
            MonthlyPrice = cancelled.Price.Amount,
            Currency = cancelled.Price.Currency,
            Status = cancelled.Status.ToString(),
            NextRenewalDate = cancelled.NextRenewalDate,
            CreatedAt = cancelled.CreatedAt
        };
    }
}
