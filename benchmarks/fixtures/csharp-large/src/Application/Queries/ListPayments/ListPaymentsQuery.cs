namespace CSharpLargeApi.Application.Queries.ListPayments;

/// <summary>
/// Query to list payments for a specific user with pagination.
/// </summary>
public sealed class ListPaymentsQuery
{
    /// <summary>
    /// Gets the ID of the user whose payments to list.
    /// </summary>
    public Guid UserId { get; }

    /// <summary>
    /// Gets the number of records to skip for pagination.
    /// </summary>
    public int Skip { get; }

    /// <summary>
    /// Gets the maximum number of records to return.
    /// </summary>
    public int Take { get; }

    /// <summary>
    /// Creates a new ListPaymentsQuery.
    /// </summary>
    public ListPaymentsQuery(Guid userId, int skip = 0, int take = 20)
    {
        UserId = userId;
        Skip = skip;
        Take = Math.Clamp(take, 1, 100);
    }
}
