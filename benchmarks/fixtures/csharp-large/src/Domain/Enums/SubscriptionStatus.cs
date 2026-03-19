namespace CSharpLargeApi.Domain.Enums;

/// <summary>
/// Represents the lifecycle status of a subscription.
/// </summary>
public enum SubscriptionStatus
{
    /// <summary>Subscription is active and in good standing.</summary>
    Active = 0,

    /// <summary>Subscription is past due but not yet cancelled.</summary>
    PastDue = 1,

    /// <summary>Subscription has been suspended due to payment failure.</summary>
    Suspended = 2,

    /// <summary>Subscription has been cancelled by the user.</summary>
    Cancelled = 3,

    /// <summary>Subscription has expired and was not renewed.</summary>
    Expired = 4,

    /// <summary>Subscription is in a trial period.</summary>
    Trial = 5
}
