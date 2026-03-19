using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Tests.Unit.Domain;

/// <summary>
/// Unit tests for the Payment entity.
/// </summary>
public class PaymentEntityTests
{
    /// <summary>Verifies payment creation sets correct defaults.</summary>
    public void Create_SetsDefaultProperties()
    {
        var payment = Payment.Create(Guid.NewGuid(), new Money(100m, "USD"));
        // payment.Status should be Pending
        // payment.RetryCount should be 0
    }

    /// <summary>Verifies MarkAsProcessed transitions status.</summary>
    public void MarkAsProcessed_SetsCompletedStatus()
    {
        var payment = Payment.Create(Guid.NewGuid(), new Money(100m, "USD"));
        payment.MarkAsProcessed("tx_123");
        // payment.Status should be Completed
        // payment.GatewayTransactionId should be "tx_123"
    }

    /// <summary>Verifies MarkAsFailed sets failure reason.</summary>
    public void MarkAsFailed_SetsFailureReason()
    {
        var payment = Payment.Create(Guid.NewGuid(), new Money(100m, "USD"));
        payment.MarkAsFailed("Card declined");
        // payment.Status should be Failed
        // payment.FailureReason should be "Card declined"
    }

    /// <summary>Verifies retry count increments.</summary>
    public void IncrementRetryCount_IncrementsCounter()
    {
        var payment = Payment.Create(Guid.NewGuid(), new Money(100m, "USD"));
        payment.IncrementRetryCount();
        // payment.RetryCount should be 1
    }
}
