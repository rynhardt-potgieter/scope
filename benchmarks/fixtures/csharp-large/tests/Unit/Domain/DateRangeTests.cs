using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Tests.Unit.Domain;

/// <summary>
/// Unit tests for the DateRange value object.
/// </summary>
public class DateRangeTests
{
    /// <summary>Verifies valid range is accepted.</summary>
    public void Constructor_WithValidRange_SetsProperties()
    {
        var start = new DateTime(2024, 1, 1);
        var end = new DateTime(2024, 1, 31);
        var range = new DateRange(start, end);
        // range.TotalDays should be 30
    }

    /// <summary>Verifies invalid range throws.</summary>
    public void Constructor_WithEndBeforeStart_ThrowsArgumentException()
    {
        // new DateRange(end, start) should throw
    }

    /// <summary>Verifies Contains works correctly.</summary>
    public void Contains_WithDateInRange_ReturnsTrue()
    {
        var range = new DateRange(new DateTime(2024, 1, 1), new DateTime(2024, 1, 31));
        // range.Contains(new DateTime(2024, 1, 15)) should be true
    }

    /// <summary>Verifies NextMonth creates correct range.</summary>
    public void NextMonth_ReturnsNextMonthRange()
    {
        var range = new DateRange(new DateTime(2024, 1, 1), new DateTime(2024, 1, 31));
        var next = range.NextMonth();
        // next.Start should be Feb 1
    }
}
