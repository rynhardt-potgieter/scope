namespace CSharpLargeApi.Application.DTOs;

/// <summary>
/// Data transfer object for subscription information.
/// </summary>
public class SubscriptionDto
{
    /// <summary>
    /// Gets or sets the subscription identifier.
    /// </summary>
    public Guid Id { get; set; }

    /// <summary>
    /// Gets or sets the user identifier.
    /// </summary>
    public Guid UserId { get; set; }

    /// <summary>
    /// Gets or sets the plan name.
    /// </summary>
    public string PlanName { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the monthly price.
    /// </summary>
    public decimal MonthlyPrice { get; set; }

    /// <summary>
    /// Gets or sets the currency code.
    /// </summary>
    public string Currency { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the subscription status.
    /// </summary>
    public string Status { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the next renewal date.
    /// </summary>
    public DateTime NextRenewalDate { get; set; }

    /// <summary>
    /// Gets or sets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; set; }
}
