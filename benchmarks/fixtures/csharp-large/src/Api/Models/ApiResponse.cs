namespace CSharpLargeApi.Api.Models;

/// <summary>
/// Standard API response wrapper for consistent response formatting.
/// </summary>
/// <typeparam name="T">The type of data contained in the response.</typeparam>
public class ApiResponse<T>
{
    /// <summary>Gets or sets whether the request was successful.</summary>
    public bool Success { get; set; }

    /// <summary>Gets or sets the response data.</summary>
    public T? Data { get; set; }

    /// <summary>Gets or sets the error message if the request failed.</summary>
    public string? Error { get; set; }

    /// <summary>Gets or sets the error code if the request failed.</summary>
    public string? ErrorCode { get; set; }

    /// <summary>Gets or sets the request trace ID for debugging.</summary>
    public string TraceId { get; set; } = Guid.NewGuid().ToString("N");

    /// <summary>Creates a successful response with data.</summary>
    public static ApiResponse<T> Ok(T data) => new() { Success = true, Data = data };

    /// <summary>Creates a failure response with error details.</summary>
    public static ApiResponse<T> Fail(string error, string errorCode = "ERROR") =>
        new() { Success = false, Error = error, ErrorCode = errorCode };
}

/// <summary>
/// Paginated API response wrapper.
/// </summary>
public class PaginatedResponse<T>
{
    /// <summary>Gets or sets the items on the current page.</summary>
    public IReadOnlyList<T> Items { get; set; } = Array.Empty<T>();

    /// <summary>Gets or sets the total number of items across all pages.</summary>
    public int TotalCount { get; set; }

    /// <summary>Gets or sets the current page number (0-based).</summary>
    public int Page { get; set; }

    /// <summary>Gets or sets the page size.</summary>
    public int PageSize { get; set; }

    /// <summary>Gets whether there are more pages after this one.</summary>
    public bool HasMore => (Page + 1) * PageSize < TotalCount;
}
