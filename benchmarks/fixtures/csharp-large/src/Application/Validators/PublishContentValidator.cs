using CSharpLargeApi.Application.Commands.PublishContent;

namespace CSharpLargeApi.Application.Validators;

/// <summary>
/// Validates PublishContentCommand instances before they reach the handler.
/// Ensures the command contains valid identifiers.
/// </summary>
public class PublishContentValidator
{
    /// <summary>
    /// Validates the given command and returns a list of validation errors.
    /// Returns an empty list if the command is valid.
    /// </summary>
    public IReadOnlyList<string> Validate(PublishContentCommand command)
    {
        var errors = new List<string>();

        if (command.ContentId == Guid.Empty)
        {
            errors.Add("ContentId is required.");
        }

        if (command.PublishedByUserId == Guid.Empty)
        {
            errors.Add("PublishedByUserId is required.");
        }

        return errors.AsReadOnly();
    }
}
