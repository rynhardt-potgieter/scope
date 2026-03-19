namespace CSharpLargeApi.Shared.Extensions;

/// <summary>
/// Extension methods for IEnumerable and collection types.
/// </summary>
public static class EnumerableExtensions
{
    /// <summary>
    /// Returns a paginated subset of the collection.
    /// </summary>
    public static IEnumerable<T> Paginate<T>(this IEnumerable<T> source, int skip, int take)
    {
        return source.Skip(skip).Take(take);
    }

    /// <summary>
    /// Splits a collection into batches of the specified size.
    /// </summary>
    public static IEnumerable<IReadOnlyList<T>> Batch<T>(this IEnumerable<T> source, int batchSize)
    {
        var batch = new List<T>(batchSize);

        foreach (var item in source)
        {
            batch.Add(item);
            if (batch.Count >= batchSize)
            {
                yield return batch.AsReadOnly();
                batch = new List<T>(batchSize);
            }
        }

        if (batch.Count > 0)
        {
            yield return batch.AsReadOnly();
        }
    }

    /// <summary>
    /// Returns distinct elements based on a key selector function.
    /// </summary>
    public static IEnumerable<T> DistinctBy<T, TKey>(this IEnumerable<T> source, Func<T, TKey> keySelector)
    {
        var seen = new HashSet<TKey>();

        foreach (var item in source)
        {
            var key = keySelector(item);
            if (seen.Add(key))
            {
                yield return item;
            }
        }
    }

    /// <summary>
    /// Executes an action for each element in the collection.
    /// </summary>
    public static void ForEach<T>(this IEnumerable<T> source, Action<T> action)
    {
        foreach (var item in source)
        {
            action(item);
        }
    }

    /// <summary>
    /// Returns true if the collection is null or empty.
    /// </summary>
    public static bool IsNullOrEmpty<T>(this IEnumerable<T>? source)
    {
        return source is null || !source.Any();
    }
}
