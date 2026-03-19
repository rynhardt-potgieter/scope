using System.Text.RegularExpressions;

namespace CSharpLargeApi.Domain.ValueObjects;

/// <summary>
/// Represents a validated phone number with country code.
/// Stored in E.164 format (e.g. "+14155552671").
/// </summary>
public sealed class PhoneNumber : IEquatable<PhoneNumber>
{
    private static readonly Regex PhonePattern = new(
        @"^\+[1-9]\d{6,14}$",
        RegexOptions.Compiled);

    /// <summary>
    /// Gets the phone number in E.164 format.
    /// </summary>
    public string Value { get; }

    /// <summary>
    /// Creates a new PhoneNumber after validating the E.164 format.
    /// </summary>
    public PhoneNumber(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Phone number cannot be empty.", nameof(value));

        var normalized = value.Replace(" ", "").Replace("-", "").Replace("(", "").Replace(")", "");

        if (!PhonePattern.IsMatch(normalized))
            throw new ArgumentException($"'{value}' is not a valid E.164 phone number.", nameof(value));

        Value = normalized;
    }

    /// <summary>
    /// Gets the country code portion of the phone number.
    /// </summary>
    public string CountryCode
    {
        get
        {
            // Simple heuristic: 1-3 digit country codes
            if (Value.StartsWith("+1") && Value.Length == 12) return "+1";
            return Value[..3];
        }
    }

    /// <summary>
    /// Determines whether this phone number equals another.
    /// </summary>
    public bool Equals(PhoneNumber? other)
    {
        if (other is null) return false;
        return Value == other.Value;
    }

    /// <inheritdoc />
    public override bool Equals(object? obj) => Equals(obj as PhoneNumber);

    /// <inheritdoc />
    public override int GetHashCode() => Value.GetHashCode();

    /// <inheritdoc />
    public override string ToString() => Value;
}
