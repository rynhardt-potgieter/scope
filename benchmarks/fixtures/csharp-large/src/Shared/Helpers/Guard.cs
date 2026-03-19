namespace CSharpLargeApi.Shared.Helpers;

/// <summary>
/// Argument validation helper methods.
/// Provides a fluent API for common parameter validation patterns.
/// </summary>
public static class Guard
{
    /// <summary>
    /// Throws ArgumentNullException if the value is null.
    /// </summary>
    public static T AgainstNull<T>(T? value, string parameterName) where T : class
    {
        if (value is null)
        {
            throw new ArgumentNullException(parameterName, $"{parameterName} cannot be null.");
        }
        return value;
    }

    /// <summary>
    /// Throws ArgumentException if the string is null or empty.
    /// </summary>
    public static string AgainstNullOrEmpty(string? value, string parameterName)
    {
        if (string.IsNullOrWhiteSpace(value))
        {
            throw new ArgumentException($"{parameterName} cannot be null or empty.", parameterName);
        }
        return value;
    }

    /// <summary>
    /// Throws ArgumentException if the GUID is empty.
    /// </summary>
    public static Guid AgainstEmptyGuid(Guid value, string parameterName)
    {
        if (value == Guid.Empty)
        {
            throw new ArgumentException($"{parameterName} cannot be an empty GUID.", parameterName);
        }
        return value;
    }

    /// <summary>
    /// Throws ArgumentOutOfRangeException if the value is not within the specified range.
    /// </summary>
    public static T AgainstOutOfRange<T>(T value, T min, T max, string parameterName) where T : IComparable<T>
    {
        if (value.CompareTo(min) < 0 || value.CompareTo(max) > 0)
        {
            throw new ArgumentOutOfRangeException(parameterName, value, $"{parameterName} must be between {min} and {max}.");
        }
        return value;
    }

    /// <summary>
    /// Throws ArgumentOutOfRangeException if the value is negative.
    /// </summary>
    public static decimal AgainstNegative(decimal value, string parameterName)
    {
        if (value < 0)
        {
            throw new ArgumentOutOfRangeException(parameterName, value, $"{parameterName} cannot be negative.");
        }
        return value;
    }

    /// <summary>
    /// Throws ArgumentOutOfRangeException if the value is zero or negative.
    /// </summary>
    public static decimal AgainstZeroOrNegative(decimal value, string parameterName)
    {
        if (value <= 0)
        {
            throw new ArgumentOutOfRangeException(parameterName, value, $"{parameterName} must be greater than zero.");
        }
        return value;
    }

    /// <summary>
    /// Throws ArgumentException if the collection is null or empty.
    /// </summary>
    public static IEnumerable<T> AgainstNullOrEmptyCollection<T>(IEnumerable<T>? value, string parameterName)
    {
        if (value is null || !value.Any())
        {
            throw new ArgumentException($"{parameterName} cannot be null or empty.", parameterName);
        }
        return value;
    }
}
