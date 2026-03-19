using System.Text.RegularExpressions;

namespace CSharpLargeApi.Domain.ValueObjects;

/// <summary>
/// Represents a validated email address.
/// Ensures the email conforms to a basic format on construction.
/// </summary>
public sealed class EmailAddress : IEquatable<EmailAddress>
{
    private static readonly Regex EmailPattern = new(
        @"^[^@\s]+@[^@\s]+\.[^@\s]+$",
        RegexOptions.Compiled | RegexOptions.IgnoreCase);

    /// <summary>
    /// Gets the validated email address string.
    /// </summary>
    public string Value { get; }

    /// <summary>
    /// Creates a new EmailAddress after validating the format.
    /// </summary>
    public EmailAddress(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Email address cannot be empty.", nameof(value));

        if (!EmailPattern.IsMatch(value))
            throw new ArgumentException($"'{value}' is not a valid email address.", nameof(value));

        Value = value.Trim().ToLowerInvariant();
    }

    /// <summary>
    /// Gets the domain portion of the email address.
    /// </summary>
    public string Domain => Value.Split('@')[1];

    /// <summary>
    /// Gets the local part (before the @) of the email address.
    /// </summary>
    public string LocalPart => Value.Split('@')[0];

    /// <summary>
    /// Determines whether this email equals another.
    /// </summary>
    public bool Equals(EmailAddress? other)
    {
        if (other is null) return false;
        return Value == other.Value;
    }

    /// <inheritdoc />
    public override bool Equals(object? obj) => Equals(obj as EmailAddress);

    /// <inheritdoc />
    public override int GetHashCode() => Value.GetHashCode();

    /// <inheritdoc />
    public override string ToString() => Value;
}
