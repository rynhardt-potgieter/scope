using System.Text.Json;
using System.Text.Json.Serialization;

namespace CSharpLargeApi.Shared.Helpers;

/// <summary>
/// Helper methods for JSON serialization and deserialization.
/// Provides consistent JSON options across the application.
/// </summary>
public static class JsonHelper
{
    private static readonly JsonSerializerOptions DefaultOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        WriteIndented = false,
        PropertyNameCaseInsensitive = true
    };

    private static readonly JsonSerializerOptions IndentedOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        WriteIndented = true,
        PropertyNameCaseInsensitive = true
    };

    /// <summary>
    /// Serializes an object to a JSON string using the default application options.
    /// </summary>
    public static string Serialize<T>(T value)
    {
        return JsonSerializer.Serialize(value, DefaultOptions);
    }

    /// <summary>
    /// Serializes an object to a pretty-printed JSON string.
    /// </summary>
    public static string SerializeIndented<T>(T value)
    {
        return JsonSerializer.Serialize(value, IndentedOptions);
    }

    /// <summary>
    /// Deserializes a JSON string to the specified type.
    /// Returns default if the input is null or empty.
    /// </summary>
    public static T? Deserialize<T>(string? json)
    {
        if (string.IsNullOrWhiteSpace(json))
            return default;

        return JsonSerializer.Deserialize<T>(json, DefaultOptions);
    }

    /// <summary>
    /// Attempts to deserialize a JSON string, returning false on failure.
    /// </summary>
    public static bool TryDeserialize<T>(string? json, out T? result)
    {
        result = default;

        if (string.IsNullOrWhiteSpace(json))
            return false;

        try
        {
            result = JsonSerializer.Deserialize<T>(json, DefaultOptions);
            return result is not null;
        }
        catch (JsonException)
        {
            return false;
        }
    }

    /// <summary>
    /// Gets the default JSON serializer options used by the application.
    /// </summary>
    public static JsonSerializerOptions GetDefaultOptions() => DefaultOptions;
}
