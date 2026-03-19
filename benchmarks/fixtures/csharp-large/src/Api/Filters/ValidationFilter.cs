namespace CSharpLargeApi.Api.Filters;

/// <summary>
/// Action filter that automatically validates request models before
/// the action method executes. Returns 400 Bad Request with validation
/// errors if the model is invalid.
/// </summary>
public class ValidationFilter
{
    /// <summary>
    /// Validates the given model and returns errors if any.
    /// </summary>
    public IReadOnlyList<ValidationError> Validate(object model)
    {
        var errors = new List<ValidationError>();

        if (model is null)
        {
            errors.Add(new ValidationError("model", "Request body is required."));
            return errors.AsReadOnly();
        }

        var properties = model.GetType().GetProperties();
        foreach (var prop in properties)
        {
            var value = prop.GetValue(model);

            if (prop.PropertyType == typeof(string) && value is string str)
            {
                if (string.IsNullOrWhiteSpace(str) && IsRequired(prop.Name))
                {
                    errors.Add(new ValidationError(prop.Name, $"{prop.Name} is required."));
                }

                if (str.Length > 10000)
                {
                    errors.Add(new ValidationError(prop.Name, $"{prop.Name} exceeds maximum length."));
                }
            }

            if (prop.PropertyType == typeof(Guid) && value is Guid guid)
            {
                if (guid == Guid.Empty && IsRequired(prop.Name))
                {
                    errors.Add(new ValidationError(prop.Name, $"{prop.Name} is required."));
                }
            }
        }

        return errors.AsReadOnly();
    }

    private static bool IsRequired(string propertyName)
    {
        // Convention: properties ending in "Id" or named "Name", "Email" are required
        return propertyName.EndsWith("Id") ||
               propertyName is "Name" or "Email" or "Title" or "Subject";
    }
}

/// <summary>
/// Represents a single validation error for a specific field.
/// </summary>
public class ValidationError
{
    /// <summary>
    /// Gets the name of the field that failed validation.
    /// </summary>
    public string Field { get; }

    /// <summary>
    /// Gets the validation error message.
    /// </summary>
    public string Message { get; }

    /// <summary>
    /// Creates a new validation error.
    /// </summary>
    public ValidationError(string field, string message)
    {
        Field = field;
        Message = message;
    }
}
