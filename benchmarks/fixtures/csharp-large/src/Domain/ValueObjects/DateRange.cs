namespace CSharpLargeApi.Domain.ValueObjects;

/// <summary>
/// Represents a date range with a start and end date.
/// Ensures that the start date is always before or equal to the end date.
/// </summary>
public sealed class DateRange : IEquatable<DateRange>
{
    /// <summary>
    /// Gets the start date of the range (inclusive).
    /// </summary>
    public DateTime Start { get; }

    /// <summary>
    /// Gets the end date of the range (inclusive).
    /// </summary>
    public DateTime End { get; }

    /// <summary>
    /// Creates a new DateRange with the specified start and end dates.
    /// </summary>
    public DateRange(DateTime start, DateTime end)
    {
        if (end < start)
            throw new ArgumentException("End date must be on or after the start date.");

        Start = start;
        End = end;
    }

    /// <summary>
    /// Gets the total number of days in this range.
    /// </summary>
    public int TotalDays => (End - Start).Days;

    /// <summary>
    /// Checks whether the given date falls within this range.
    /// </summary>
    public bool Contains(DateTime date)
    {
        return date >= Start && date <= End;
    }

    /// <summary>
    /// Checks whether this range overlaps with another range.
    /// </summary>
    public bool Overlaps(DateRange other)
    {
        return Start <= other.End && End >= other.Start;
    }

    /// <summary>
    /// Creates a new DateRange for the next month starting from the end of this range.
    /// </summary>
    public DateRange NextMonth()
    {
        var newStart = End.AddDays(1);
        var newEnd = newStart.AddMonths(1).AddDays(-1);
        return new DateRange(newStart, newEnd);
    }

    /// <summary>
    /// Determines whether this date range equals another.
    /// </summary>
    public bool Equals(DateRange? other)
    {
        if (other is null) return false;
        return Start == other.Start && End == other.End;
    }

    /// <inheritdoc />
    public override bool Equals(object? obj) => Equals(obj as DateRange);

    /// <inheritdoc />
    public override int GetHashCode() => HashCode.Combine(Start, End);

    /// <inheritdoc />
    public override string ToString() => $"{Start:yyyy-MM-dd} to {End:yyyy-MM-dd}";
}
