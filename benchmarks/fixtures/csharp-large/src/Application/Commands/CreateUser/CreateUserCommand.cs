using CSharpLargeApi.Domain.Enums;

namespace CSharpLargeApi.Application.Commands.CreateUser;

/// <summary>
/// Command to create a new user account in the system.
/// Contains all required registration information.
/// </summary>
public sealed class CreateUserCommand
{
    /// <summary>
    /// Gets the desired display name for the new user.
    /// </summary>
    public string DisplayName { get; }

    /// <summary>
    /// Gets the email address for the new user.
    /// </summary>
    public string Email { get; }

    /// <summary>
    /// Gets the plain-text password to be hashed before storage.
    /// </summary>
    public string Password { get; }

    /// <summary>
    /// Gets the role to assign to the new user.
    /// </summary>
    public UserRole Role { get; }

    /// <summary>
    /// Gets the optional phone number for the new user.
    /// </summary>
    public string? PhoneNumber { get; }

    /// <summary>
    /// Creates a new CreateUserCommand.
    /// </summary>
    public CreateUserCommand(string displayName, string email, string password, UserRole role, string? phoneNumber = null)
    {
        DisplayName = displayName;
        Email = email;
        Password = password;
        Role = role;
        PhoneNumber = phoneNumber;
    }
}
