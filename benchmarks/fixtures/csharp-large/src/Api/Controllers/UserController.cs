using CSharpLargeApi.Application.Commands.CreateUser;
using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Application.Mappings;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Api.Controllers;

/// <summary>
/// API controller for user management operations.
/// Provides endpoints for CRUD operations on user accounts.
/// </summary>
public class UserController
{
    private readonly IUserService _userService;

    /// <summary>
    /// Initializes the controller with the user service.
    /// </summary>
    public UserController(IUserService userService)
    {
        _userService = userService ?? throw new ArgumentNullException(nameof(userService));
    }

    /// <summary>
    /// Creates a new user account.
    /// POST /api/users
    /// </summary>
    public async Task<UserDto> CreateUser(string displayName, string email, string password, UserRole role, CancellationToken cancellationToken = default)
    {
        var emailAddress = new EmailAddress(email);
        var user = await _userService.CreateUserAsync(displayName, emailAddress, password, role, cancellationToken);
        return UserMappingProfile.ToDto(user);
    }

    /// <summary>
    /// Retrieves a user by their identifier.
    /// GET /api/users/{id}
    /// </summary>
    public async Task<UserDto> GetUser(Guid userId, CancellationToken cancellationToken = default)
    {
        var user = await _userService.GetUserAsync(userId, cancellationToken);
        if (user is null)
        {
            throw new EntityNotFoundException("User", userId);
        }
        return UserMappingProfile.ToDto(user);
    }

    /// <summary>
    /// Updates a user's profile information.
    /// PUT /api/users/{id}
    /// </summary>
    public async Task<UserDto> UpdateUser(Guid userId, string displayName, string? phoneNumber, CancellationToken cancellationToken = default)
    {
        var phone = phoneNumber is not null ? new PhoneNumber(phoneNumber) : null;
        var user = await _userService.UpdateUserAsync(userId, displayName, phone, cancellationToken);
        return UserMappingProfile.ToDto(user);
    }

    /// <summary>
    /// Lists users with pagination.
    /// GET /api/users?skip={skip}&take={take}
    /// </summary>
    public async Task<IReadOnlyList<UserDto>> ListUsers(int skip = 0, int take = 20, CancellationToken cancellationToken = default)
    {
        var users = await _userService.ListUsersAsync(skip, take, cancellationToken);
        return UserMappingProfile.ToDtoList(users);
    }

    /// <summary>
    /// Deactivates a user account.
    /// DELETE /api/users/{id}
    /// </summary>
    public async Task DeactivateUser(Guid userId, CancellationToken cancellationToken = default)
    {
        await _userService.DeactivateUserAsync(userId, cancellationToken);
    }
}
