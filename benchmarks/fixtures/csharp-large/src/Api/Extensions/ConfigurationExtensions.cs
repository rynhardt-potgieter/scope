namespace CSharpLargeApi.Api.Extensions;

/// <summary>
/// Extension methods for reading and validating application configuration.
/// Provides typed access to configuration sections with validation.
/// </summary>
public static class ConfigurationExtensions
{
    /// <summary>
    /// Reads a required configuration value and throws if missing.
    /// </summary>
    public static string GetRequired(this Dictionary<string, string> config, string key)
    {
        if (!config.TryGetValue(key, out var value) || string.IsNullOrWhiteSpace(value))
        {
            throw new InvalidOperationException($"Required configuration key '{key}' is missing or empty.");
        }
        return value;
    }

    /// <summary>
    /// Reads an optional configuration value with a default fallback.
    /// </summary>
    public static string GetOptional(this Dictionary<string, string> config, string key, string defaultValue)
    {
        if (config.TryGetValue(key, out var value) && !string.IsNullOrWhiteSpace(value))
        {
            return value;
        }
        return defaultValue;
    }

    /// <summary>
    /// Reads a configuration value as an integer.
    /// </summary>
    public static int GetInt(this Dictionary<string, string> config, string key, int defaultValue)
    {
        if (config.TryGetValue(key, out var value) && int.TryParse(value, out var intValue))
        {
            return intValue;
        }
        return defaultValue;
    }

    /// <summary>
    /// Validates that all required configuration keys are present.
    /// </summary>
    public static void ValidateRequired(this Dictionary<string, string> config, params string[] requiredKeys)
    {
        var missing = requiredKeys
            .Where(k => !config.ContainsKey(k) || string.IsNullOrWhiteSpace(config[k]))
            .ToList();

        if (missing.Count > 0)
        {
            throw new InvalidOperationException(
                $"Missing required configuration keys: {string.Join(", ", missing)}");
        }
    }
}
