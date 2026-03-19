using CSharpLargeApi.Domain.Enums;

namespace CSharpLargeApi.Application.Queries.SearchContent;

/// <summary>
/// Query to search content by text with optional status filtering.
/// </summary>
public sealed class SearchContentQuery
{
    /// <summary>
    /// Gets the search text to match against titles and body content.
    /// </summary>
    public string SearchText { get; }

    /// <summary>
    /// Gets the optional status filter. If null, searches all statuses.
    /// </summary>
    public ContentStatus? Status { get; }

    /// <summary>
    /// Gets the number of records to skip for pagination.
    /// </summary>
    public int Skip { get; }

    /// <summary>
    /// Gets the maximum number of records to return.
    /// </summary>
    public int Take { get; }

    /// <summary>
    /// Creates a new SearchContentQuery.
    /// </summary>
    public SearchContentQuery(string searchText, ContentStatus? status = null, int skip = 0, int take = 20)
    {
        SearchText = searchText;
        Status = status;
        Skip = skip;
        Take = Math.Clamp(take, 1, 100);
    }
}
