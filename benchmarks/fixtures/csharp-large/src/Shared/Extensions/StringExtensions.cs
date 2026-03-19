using System.Globalization;
using System.Text.RegularExpressions;

namespace CSharpLargeApi.Shared.Extensions;

/// <summary>
/// Extension methods for string manipulation and formatting.
/// </summary>
public static class StringExtensions
{
    /// <summary>
    /// Converts a string to a URL-friendly slug.
    /// Lowercases, replaces spaces with hyphens, and removes non-alphanumeric characters.
    /// </summary>
    public static string ToSlug(this string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            return string.Empty;

        var slug = value.ToLowerInvariant().Trim();
        slug = Regex.Replace(slug, @"\s+", "-");
        slug = Regex.Replace(slug, @"[^a-z0-9\-]", "");
        slug = Regex.Replace(slug, @"-+", "-");
        slug = slug.Trim('-');

        return slug;
    }

    /// <summary>
    /// Truncates a string to the specified maximum length, adding an ellipsis if truncated.
    /// </summary>
    public static string Truncate(this string value, int maxLength, string suffix = "...")
    {
        if (string.IsNullOrEmpty(value) || value.Length <= maxLength)
            return value;

        return value[..(maxLength - suffix.Length)] + suffix;
    }

    /// <summary>
    /// Converts a string to title case.
    /// </summary>
    public static string ToTitleCase(this string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            return value;

        return CultureInfo.CurrentCulture.TextInfo.ToTitleCase(value.ToLowerInvariant());
    }

    /// <summary>
    /// Masks a string, showing only the first and last N characters.
    /// Useful for displaying partial email addresses or payment tokens.
    /// </summary>
    public static string Mask(this string value, int visibleStart = 3, int visibleEnd = 3, char maskChar = '*')
    {
        if (string.IsNullOrEmpty(value) || value.Length <= visibleStart + visibleEnd)
            return value;

        var start = value[..visibleStart];
        var end = value[^visibleEnd..];
        var masked = new string(maskChar, value.Length - visibleStart - visibleEnd);

        return start + masked + end;
    }

    /// <summary>
    /// Checks whether a string is a valid email format.
    /// </summary>
    public static bool IsValidEmail(this string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            return false;

        return Regex.IsMatch(value, @"^[^@\s]+@[^@\s]+\.[^@\s]+$", RegexOptions.IgnoreCase);
    }
}
