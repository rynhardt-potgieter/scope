using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Application.Commands.ProcessPayment;

/// <summary>
/// Handles the ProcessPaymentCommand by orchestrating validation,
/// payment processing, and response mapping.
/// This is a CQRS command handler — caller #4 of ProcessPayment.
/// </summary>
public class ProcessPaymentHandler
{
    private readonly IPaymentService _paymentService;
    private readonly IUserService _userService;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the handler with required dependencies.
    /// </summary>
    public ProcessPaymentHandler(
        IPaymentService paymentService,
        IUserService userService,
        INotificationService notificationService)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _userService = userService ?? throw new ArgumentNullException(nameof(userService));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Handles the command by validating the user exists, processing the payment,
    /// and sending a confirmation notification.
    /// </summary>
    public async Task<PaymentDto> Handle(ProcessPaymentCommand command, CancellationToken cancellationToken)
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
                "Cannot process payment for a deactivated user account.",
                "User");
        }

        var isValidMethod = await _paymentService.ValidatePaymentMethod(
            command.PaymentMethodToken, cancellationToken);

        if (!isValidMethod)
        {
            throw new BusinessRuleException(
                "ValidPaymentMethodRequired",
                "The provided payment method is invalid or expired.",
                "Payment");
        }

        var amount = new Money(command.Amount, command.Currency);

        // Caller #4: ProcessPaymentHandler.Handle calls IPaymentService.ProcessPayment
        var payment = await _paymentService.ProcessPayment(
            command.UserId,
            amount,
            command.PaymentMethodToken,
            cancellationToken);

        await _notificationService.SendEmailAsync(
            command.UserId,
            "Payment Confirmation",
            $"Your payment of {amount} has been processed successfully. Transaction ID: {payment.Id}",
            cancellationToken);

        return new PaymentDto
        {
            Id = payment.Id,
            UserId = payment.UserId,
            Amount = payment.Amount.Amount,
            Currency = payment.Amount.Currency,
            Status = payment.Status.ToString(),
            GatewayTransactionId = payment.GatewayTransactionId,
            CreatedAt = payment.CreatedAt
        };
    }
}
