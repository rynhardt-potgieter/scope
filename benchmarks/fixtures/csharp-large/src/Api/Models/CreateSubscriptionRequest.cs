namespace CSharpLargeApi.Api.Models;

/// <summary>
/// Request model for creating a new subscription.
/// Binds to the JSON body of POST /api/subscriptions.
/// </summary>
public class CreateSubscriptionRequest
{
    /// <summary>Gets or sets the user ID.</summary>
    public Guid UserId { get; set; }

    /// <summary>Gets or sets the plan name.</summary>
    public string PlanName { get; set; } = string.Empty;

    /// <summary>Gets or sets the monthly price.</summary>
    public decimal MonthlyPrice { get; set; }

    /// <summary>Gets or sets the currency code.</summary>
    public string Currency { get; set; } = "USD";

    /// <summary>Gets or sets the payment method token.</summary>
    public string PaymentMethodToken { get; set; } = string.Empty;
}
