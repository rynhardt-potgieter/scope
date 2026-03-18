using CSharpApi.Payments;
using CSharpApi.Users;

namespace CSharpApi.Controllers;

/// <summary>
/// Controller managing subscription lifecycle and recurring payments.
/// Handles renewals, upgrades, and subscription cancellations.
/// </summary>
public class SubscriptionController
{
    private readonly IPaymentService _paymentService;
    private readonly UserService _userService;

    /// <summary>
    /// Initializes a new SubscriptionController with the required services.
    /// </summary>
    public SubscriptionController(IPaymentService paymentService, UserService userService)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _userService = userService ?? throw new ArgumentNullException(nameof(userService));
    }

    /// <summary>
    /// Renew an existing subscription by charging the saved card.
    /// Verifies the user is still active before processing the renewal payment.
    /// </summary>
    public PaymentResult Renew(string userId, decimal amount, CardDetails card)
    {
        var user = _userService.GetUser(userId);
        if (!user.IsActive)
        {
            return new PaymentResult
            {
                TransactionId = string.Empty,
                Status = "failed",
                Amount = 0,
                Currency = "USD",
                Timestamp = DateTime.UtcNow,
                ErrorMessage = $"User {userId} is not active"
            };
        }

        // Call ProcessPayment — caller #4
        var result = _paymentService.ProcessPayment(amount, userId, card);
        return result;
    }

    /// <summary>
    /// Upgrade a subscription to a higher tier, charging the price difference.
    /// Calculates the prorated amount and charges immediately.
    /// </summary>
    public PaymentResult Upgrade(string userId, decimal priceDifference, CardDetails card)
    {
        // Call ProcessPayment — caller #5
        var result = _paymentService.ProcessPayment(priceDifference, userId, card);
        return result;
    }

    /// <summary>
    /// Cancel a subscription and refund the remaining balance.
    /// </summary>
    public PaymentResult Cancel(string transactionId)
    {
        return _paymentService.RefundPayment(transactionId);
    }
}
