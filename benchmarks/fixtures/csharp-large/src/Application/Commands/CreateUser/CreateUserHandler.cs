using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Application.Commands.CreateUser;

/// <summary>
/// Handles the CreateUserCommand by validating uniqueness,
/// hashing the password, creating the user, and sending a welcome email.
/// </summary>
public class CreateUserHandler
{
    private readonly IUserService _userService;
    private readonly INotificationService _notificationService;

    /// <summary>
    /// Initializes the handler with required dependencies.
    /// </summary>
    public CreateUserHandler(IUserService userService, INotificationService notificationService)
    {
        _userService = userService ?? throw new ArgumentNullException(nameof(userService));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
    }

    /// <summary>
    /// Handles the command by checking for duplicate emails,
    /// creating the user, and sending a welcome notification.
    /// </summary>
    public async Task<UserDto> Handle(CreateUserCommand command, CancellationToken cancellationToken)
    {
        var existingUser = await _userService.GetUserByEmailAsync(command.Email, cancellationToken);
        if (existingUser is not null)
        {
            throw new BusinessRuleException(
                "UniqueEmailRequired",
                $"A user with email '{command.Email}' already exists.",
                "User");
        }

        var email = new EmailAddress(command.Email);

        var user = await _userService.CreateUserAsync(
            command.DisplayName,
            email,
            command.Password,
            command.Role,
            cancellationToken);

        await _notificationService.SendEmailAsync(
            user.Id,
            "Welcome to the Platform",
            $"Hi {command.DisplayName}, your account has been created successfully. Please verify your email to get started.",
            cancellationToken);

        return new UserDto
        {
            Id = user.Id,
            DisplayName = user.DisplayName,
            Email = user.Email.Value,
            Role = user.Role.ToString(),
            IsActive = user.IsActive,
            CreatedAt = user.CreatedAt
        };
    }
}
