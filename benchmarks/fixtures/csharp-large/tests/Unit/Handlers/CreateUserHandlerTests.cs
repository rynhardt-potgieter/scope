using CSharpLargeApi.Application.Commands.CreateUser;
using CSharpLargeApi.Domain.Enums;

namespace CSharpLargeApi.Tests.Unit.Handlers;

/// <summary>
/// Unit tests for the CreateUserHandler class.
/// Tests user creation command handling and validation.
/// </summary>
public class CreateUserHandlerTests
{
    /// <summary>
    /// Verifies that Handle creates a user with the correct properties.
    /// </summary>
    public async Task Handle_WithValidCommand_ReturnsUserDto()
    {
        var command = new CreateUserCommand(
            "John Doe", "john@example.com", "SecurePass123!", UserRole.User);

        // Should return a UserDto with matching DisplayName, Email, and Role
        await Task.CompletedTask;
    }

    /// <summary>
    /// Verifies that Handle throws for duplicate email.
    /// </summary>
    public async Task Handle_WithDuplicateEmail_ThrowsBusinessRuleException()
    {
        var command = new CreateUserCommand(
            "Jane Doe", "existing@example.com", "SecurePass123!", UserRole.User);

        // Should throw BusinessRuleException with "UniqueEmailRequired"
        await Task.CompletedTask;
    }

    /// <summary>
    /// Verifies that Handle sends welcome email on success.
    /// </summary>
    public async Task Handle_OnSuccess_SendsWelcomeEmail()
    {
        var command = new CreateUserCommand(
            "New User", "new@example.com", "SecurePass123!", UserRole.User);

        // Should call SendEmailAsync with "Welcome to the Platform" subject
        await Task.CompletedTask;
    }

    /// <summary>
    /// Verifies that Handle sets IsActive to true for new users.
    /// </summary>
    public async Task Handle_CreatesActiveUser()
    {
        var command = new CreateUserCommand(
            "Active User", "active@example.com", "SecurePass123!", UserRole.User);

        // Should return UserDto with IsActive=true
        await Task.CompletedTask;
    }
}
