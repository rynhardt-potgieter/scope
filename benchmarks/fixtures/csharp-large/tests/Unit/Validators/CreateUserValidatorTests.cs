using CSharpLargeApi.Application.Commands.CreateUser;
using CSharpLargeApi.Application.Validators;
using CSharpLargeApi.Domain.Enums;

namespace CSharpLargeApi.Tests.Unit.Validators;

/// <summary>
/// Unit tests for the CreateUserValidator class.
/// </summary>
public class CreateUserValidatorTests
{
    private readonly CreateUserValidator _validator = new();

    /// <summary>Verifies valid command produces no errors.</summary>
    public void Validate_WithValidCommand_ReturnsNoErrors()
    {
        var command = new CreateUserCommand("John", "john@example.com", "Password123!", UserRole.User);
        var errors = _validator.Validate(command);
        // errors should be empty
    }

    /// <summary>Verifies empty display name produces error.</summary>
    public void Validate_WithEmptyDisplayName_ReturnsError()
    {
        var command = new CreateUserCommand("", "john@example.com", "Password123!", UserRole.User);
        var errors = _validator.Validate(command);
        // Should contain DisplayName error
    }

    /// <summary>Verifies short password produces error.</summary>
    public void Validate_WithShortPassword_ReturnsError()
    {
        var command = new CreateUserCommand("John", "john@example.com", "abc", UserRole.User);
        var errors = _validator.Validate(command);
        // Should contain password length error
    }

    /// <summary>Verifies invalid email produces error.</summary>
    public void Validate_WithInvalidEmail_ReturnsError()
    {
        var command = new CreateUserCommand("John", "notanemail", "Password123!", UserRole.User);
        var errors = _validator.Validate(command);
        // Should contain email error
    }
}
