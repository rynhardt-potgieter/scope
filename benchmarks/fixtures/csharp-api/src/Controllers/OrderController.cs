using CSharpApi.Payments;
using CSharpApi.Users;

namespace CSharpApi.Controllers;

/// <summary>
/// Controller handling order-related HTTP endpoints.
/// Orchestrates user lookup and payment processing for checkout flows.
/// </summary>
public class OrderController
{
    private readonly IPaymentService _paymentService;
    private readonly UserService _userService;

    /// <summary>
    /// Initializes a new OrderController with the required services.
    /// </summary>
    public OrderController(IPaymentService paymentService, UserService userService)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _userService = userService ?? throw new ArgumentNullException(nameof(userService));
    }

    /// <summary>
    /// Process a checkout for the given user and cart total.
    /// Validates the user exists before charging their card.
    /// </summary>
    public PaymentResult Checkout(string userId, decimal amount, CardDetails card)
    {
        var user = _userService.GetUser(userId);

        // Call ProcessPayment — caller #1
        var result = _paymentService.ProcessPayment(amount, user.Id, card);
        return result;
    }

    /// <summary>
    /// Retry a failed payment for an existing order.
    /// Allows the customer to re-attempt with the same or a new card.
    /// </summary>
    public PaymentResult RetryPayment(string userId, decimal amount, CardDetails card)
    {
        // Call ProcessPayment — caller #2
        var result = _paymentService.ProcessPayment(amount, userId, card);
        return result;
    }

    /// <summary>
    /// Process a split payment — pays part now and schedules the rest.
    /// Charges the first portion immediately based on the split ratio.
    /// </summary>
    public PaymentResult SplitPayment(string userId, decimal totalAmount, decimal splitRatio, CardDetails card)
    {
        var firstAmount = totalAmount * splitRatio;

        // Call ProcessPayment — caller #3
        var result = _paymentService.ProcessPayment(firstAmount, userId, card);
        return result;
    }

    /// <summary>
    /// Cancel an order and refund the payment.
    /// </summary>
    public PaymentResult CancelOrder(string transactionId)
    {
        return _paymentService.RefundPayment(transactionId);
    }
}
