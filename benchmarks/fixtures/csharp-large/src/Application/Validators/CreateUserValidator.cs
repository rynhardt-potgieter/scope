using CSharpLargeApi.Application.Commands.CreateUser;

namespace CSharpLargeApi.Application.Validators;

/// <summary>
/// Validates CreateUserCommand instances before they reach the handler.
/// Ensures all required fields are present and properly formatted.
/// </summary>
public class CreateUserValidator
{
    private const int MinPasswordLength = 8;
    private const int MaxDisplayNameLength = 100;

    /// <summary>
    /// Validates the given command and returns a list of validation errors.
    /// Returns an empty list if the command is valid.
    /// </summary>
    public IReadOnlyList<string> Validate(CreateUserCommand command)
    {
        var errors = new List<string>();

        if (string.IsNullOrWhiteSpace(command.DisplayName))
        {
            errors.Add("DisplayName is required.");
        }
        else if (command.DisplayName.Length > MaxDisplayNameLength)
        {
            errors.Add($"DisplayName must be {MaxDisplayNameLength} characters or fewer.");
        }

        if (string.IsNullOrWhiteSpace(command.Email))
        {
            errors.Add("Email is required.");
        }
        else if (!command.Email.Contains('@'))
        {
            errors.Add("Email must be a valid email address.");
        }

        if (string.IsNullOrWhiteSpace(command.Password))
        {
            errors.Add("Password is required.");
        }
        else if (command.Password.Length < MinPasswordLength)
        {
            errors.Add($"Password must be at least {MinPasswordLength} characters.");
        }

        if (!Enum.IsDefined(command.Role))
        {
            errors.Add("Role must be a valid UserRole value.");
        }

        return errors.AsReadOnly();
    }
}
