namespace CSharpLargeApi.Application.Commands.CreateSubscription;

/// <summary>
/// Command to create a new subscription for a user.
/// Sets up recurring billing on the specified plan.
/// </summary>
public sealed class CreateSubscriptionCommand
{
    /// <summary>
    /// Gets the ID of the user subscribing.
    /// </summary>
    public Guid UserId { get; }

    /// <summary>
    /// Gets the plan name to subscribe to.
    /// </summary>
    public string PlanName { get; }

    /// <summary>
    /// Gets the monthly price for the subscription.
    /// </summary>
    public decimal MonthlyPrice { get; }

    /// <summary>
    /// Gets the currency for billing.
    /// </summary>
    public string Currency { get; }

    /// <summary>
    /// Gets the tokenized payment method for recurring charges.
    /// </summary>
    public string PaymentMethodToken { get; }

    /// <summary>
    /// Creates a new CreateSubscriptionCommand.
    /// </summary>
    public CreateSubscriptionCommand(Guid userId, string planName, decimal monthlyPrice, string currency, string paymentMethodToken)
    {
        UserId = userId;
        PlanName = planName;
        MonthlyPrice = monthlyPrice;
        Currency = currency;
        PaymentMethodToken = paymentMethodToken;
    }
}
