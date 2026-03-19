using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Application.Mappings;

/// <summary>
/// Provides mapping methods between Payment entities and PaymentDto objects.
/// Centralizes the mapping logic to avoid duplication across handlers.
/// </summary>
public static class PaymentProfile
{
    /// <summary>
    /// Maps a Payment entity to a PaymentDto.
    /// </summary>
    public static PaymentDto ToDto(Payment payment)
    {
        return new PaymentDto
        {
            Id = payment.Id,
            UserId = payment.UserId,
            Amount = payment.Amount.Amount,
            Currency = payment.Amount.Currency,
            Status = payment.Status.ToString(),
            GatewayTransactionId = payment.GatewayTransactionId,
            FailureReason = payment.FailureReason,
            CreatedAt = payment.CreatedAt
        };
    }

    /// <summary>
    /// Maps a collection of Payment entities to PaymentDto objects.
    /// </summary>
    public static IReadOnlyList<PaymentDto> ToDtoList(IEnumerable<Payment> payments)
    {
        return payments.Select(ToDto).ToList().AsReadOnly();
    }
}
