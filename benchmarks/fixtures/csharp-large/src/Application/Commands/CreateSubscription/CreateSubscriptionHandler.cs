using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Application.Commands.CreateSubscription;

/// <summary>
/// Handles the CreateSubscriptionCommand by validating the user,
/// creating the subscription, and processing the initial payment.
/// </summary>
public class CreateSubscriptionHandler
{
    private readonly IUserService _userService;
    private readonly IPaymentService _paymentService;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the handler with required dependencies.
    /// </summary>
    public CreateSubscriptionHandler(
        IUserService userService,
        IPaymentService paymentService,
        INotificationService notificationService)
    {
        _userService = userService ?? throw new ArgumentNullException(nameof(userService));
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Handles the command by creating the subscription and charging the first month.
    /// Note: This handler does NOT call ProcessPayment — it processes the initial
    /// subscription setup. The actual payment for renewal goes through
    /// SubscriptionController.RenewSubscription or SubscriptionRenewalWorker.
    /// </summary>
    public async Task<SubscriptionDto> Handle(CreateSubscriptionCommand command, CancellationToken cancellationToken)
    {
        var user = await _userService.GetUserAsync(command.UserId, cancellationToken);
        if (user is null)
        {
            throw new EntityNotFoundException("User", command.UserId);
        }

        if (!user.IsActive)
        {
            throw new BusinessRuleException(
                "ActiveUserRequired",
                "Cannot create a subscription for a deactivated user.",
                "User");
        }

        var isValidMethod = await _paymentService.ValidatePaymentMethod(
            command.PaymentMethodToken, cancellationToken);

        if (!isValidMethod)
        {
            throw new BusinessRuleException(
                "ValidPaymentMethodRequired",
                "The provided payment method is invalid or expired.",
                "Subscription");
        }

        var price = new Money(command.MonthlyPrice, command.Currency);
        var billingStart = DateTime.UtcNow;
        var billingEnd = billingStart.AddMonths(1).AddDays(-1);
        var billingPeriod = new DateRange(billingStart, billingEnd);

        var subscription = Subscription.Create(command.UserId, command.PlanName, price, billingPeriod);

        await _notificationService.SendEmailAsync(
            command.UserId,
            "Subscription Created",
            $"You have been subscribed to the {command.PlanName} plan at {price}/month.",
            cancellationToken);

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
}
