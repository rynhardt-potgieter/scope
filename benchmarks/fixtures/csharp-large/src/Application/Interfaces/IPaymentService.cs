using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Application.Interfaces;

/// <summary>
/// Defines the contract for payment processing operations.
/// Implementations handle gateway communication, transaction recording,
/// and post-payment workflows like notifications and invoice updates.
/// </summary>
public interface IPaymentService
{
    /// <summary>
    /// Processes a payment for the specified user and amount.
    /// This is the primary entry point for all payment flows in the system.
    /// Validates the request, charges via the gateway, persists the transaction,
    /// and sends confirmation notifications.
    /// </summary>
    Task<Payment> ProcessPayment(Guid userId, Money amount, string paymentMethodToken, CancellationToken cancellationToken = default);

    /// <summary>
    /// Refunds a previously completed payment in full.
    /// </summary>
    Task<Payment> RefundPayment(Guid paymentId, string reason, CancellationToken cancellationToken = default);

    /// <summary>
    /// Validates that a payment method token is still valid and can be charged.
    /// </summary>
    Task<bool> ValidatePaymentMethod(string paymentMethodToken, CancellationToken cancellationToken = default);

    /// <summary>
    /// Retrieves a payment by its unique identifier.
    /// </summary>
    Task<Payment?> GetPaymentAsync(Guid paymentId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Lists all payments for a given user, ordered by creation date descending.
    /// </summary>
    Task<IReadOnlyList<Payment>> GetPaymentsByUserAsync(Guid userId, int skip = 0, int take = 20, CancellationToken cancellationToken = default);
}
