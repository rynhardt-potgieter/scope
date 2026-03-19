namespace CSharpLargeApi.Shared.Extensions;

/// <summary>
/// Extension methods for DateTime manipulation and formatting.
/// </summary>
public static class DateTimeExtensions
{
    /// <summary>
    /// Returns the start of the day (midnight) for the given date.
    /// </summary>
    public static DateTime StartOfDay(this DateTime date)
    {
        return date.Date;
    }

    /// <summary>
    /// Returns the end of the day (23:59:59.999) for the given date.
    /// </summary>
    public static DateTime EndOfDay(this DateTime date)
    {
        return date.Date.AddDays(1).AddMilliseconds(-1);
    }

    /// <summary>
    /// Returns the start of the month for the given date.
    /// </summary>
    public static DateTime StartOfMonth(this DateTime date)
    {
        return new DateTime(date.Year, date.Month, 1, 0, 0, 0, date.Kind);
    }

    /// <summary>
    /// Returns the end of the month for the given date.
    /// </summary>
    public static DateTime EndOfMonth(this DateTime date)
    {
        return date.StartOfMonth().AddMonths(1).AddMilliseconds(-1);
    }

    /// <summary>
    /// Checks whether the date falls on a business day (Monday-Friday).
    /// </summary>
    public static bool IsBusinessDay(this DateTime date)
    {
        return date.DayOfWeek != DayOfWeek.Saturday && date.DayOfWeek != DayOfWeek.Sunday;
    }

    /// <summary>
    /// Returns a human-readable relative time string (e.g. "2 hours ago", "in 3 days").
    /// </summary>
    public static string ToRelativeTime(this DateTime date)
    {
        var now = DateTime.UtcNow;
        var diff = now - date;

        if (diff.TotalSeconds < 60)
            return "just now";
        if (diff.TotalMinutes < 60)
            return $"{(int)diff.TotalMinutes} minutes ago";
        if (diff.TotalHours < 24)
            return $"{(int)diff.TotalHours} hours ago";
        if (diff.TotalDays < 30)
            return $"{(int)diff.TotalDays} days ago";
        if (diff.TotalDays < 365)
            return $"{(int)(diff.TotalDays / 30)} months ago";

        return $"{(int)(diff.TotalDays / 365)} years ago";
    }

    /// <summary>
    /// Formats the date in ISO 8601 format suitable for API responses.
    /// </summary>
    public static string ToIso8601(this DateTime date)
    {
        return date.ToString("yyyy-MM-ddTHH:mm:ssZ");
    }
}
