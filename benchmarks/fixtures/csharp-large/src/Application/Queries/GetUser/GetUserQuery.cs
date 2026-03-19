namespace CSharpLargeApi.Application.Queries.GetUser;

/// <summary>
/// Query to retrieve a single user by their unique identifier.
/// </summary>
public sealed class GetUserQuery
{
    /// <summary>
    /// Gets the ID of the user to retrieve.
    /// </summary>
    public Guid UserId { get; }

    /// <summary>
    /// Creates a new GetUserQuery.
    /// </summary>
    public GetUserQuery(Guid userId)
    {
        UserId = userId;
    }
}
