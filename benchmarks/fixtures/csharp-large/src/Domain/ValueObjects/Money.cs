namespace CSharpLargeApi.Domain.ValueObjects;

/// <summary>
/// Represents a monetary amount with an associated currency.
/// Immutable value object that ensures currency-safe arithmetic.
/// </summary>
public sealed class Money : IEquatable<Money>
{
    /// <summary>
    /// Gets the decimal amount.
    /// </summary>
    public decimal Amount { get; }

    /// <summary>
    /// Gets the ISO 4217 currency code (e.g. "USD", "EUR").
    /// </summary>
    public string Currency { get; }

    /// <summary>
    /// Creates a new Money value with the specified amount and currency.
    /// </summary>
    public Money(decimal amount, string currency)
    {
        if (string.IsNullOrWhiteSpace(currency))
            throw new ArgumentException("Currency code is required.", nameof(currency));

        Amount = amount;
        Currency = currency.ToUpperInvariant();
    }

    /// <summary>
    /// Adds two Money values. Both must share the same currency.
    /// </summary>
    public Money Add(Money other)
    {
        EnsureSameCurrency(other);
        return new Money(Amount + other.Amount, Currency);
    }

    /// <summary>
    /// Subtracts another Money value. Both must share the same currency.
    /// </summary>
    public Money Subtract(Money other)
    {
        EnsureSameCurrency(other);
        return new Money(Amount - other.Amount, Currency);
    }

    /// <summary>
    /// Multiplies this Money value by a scalar factor.
    /// </summary>
    public Money Multiply(decimal factor)
    {
        return new Money(Amount * factor, Currency);
    }

    /// <summary>
    /// Returns a zero-amount Money value in the specified currency.
    /// </summary>
    public static Money Zero(string currency) => new(0m, currency);

    /// <summary>
    /// Determines whether this Money is equal to another.
    /// </summary>
    public bool Equals(Money? other)
    {
        if (other is null) return false;
        return Amount == other.Amount && Currency == other.Currency;
    }

    /// <inheritdoc />
    public override bool Equals(object? obj) => Equals(obj as Money);

    /// <inheritdoc />
    public override int GetHashCode() => HashCode.Combine(Amount, Currency);

    /// <inheritdoc />
    public override string ToString() => $"{Amount:F2} {Currency}";

    private void EnsureSameCurrency(Money other)
    {
        if (Currency != other.Currency)
            throw new InvalidOperationException($"Cannot combine {Currency} with {other.Currency}.");
    }
}
