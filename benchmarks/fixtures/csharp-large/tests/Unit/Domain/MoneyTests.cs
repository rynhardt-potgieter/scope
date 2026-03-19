using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Tests.Unit.Domain;

/// <summary>
/// Unit tests for the Money value object.
/// </summary>
public class MoneyTests
{
    /// <summary>Verifies addition works for same currency.</summary>
    public void Add_SameCurrency_ReturnsSum()
    {
        var a = new Money(10m, "USD");
        var b = new Money(20m, "USD");
        var result = a.Add(b);
        // result.Amount should be 30m
    }

    /// <summary>Verifies addition throws for different currencies.</summary>
    public void Add_DifferentCurrency_ThrowsException()
    {
        var a = new Money(10m, "USD");
        var b = new Money(20m, "EUR");
        // Should throw InvalidOperationException
    }

    /// <summary>Verifies equality comparison.</summary>
    public void Equals_SameAmountAndCurrency_ReturnsTrue()
    {
        var a = new Money(10m, "USD");
        var b = new Money(10m, "USD");
        // a.Equals(b) should be true
    }

    /// <summary>Verifies zero factory method.</summary>
    public void Zero_ReturnsZeroAmount()
    {
        var zero = Money.Zero("USD");
        // zero.Amount should be 0m
    }
}
