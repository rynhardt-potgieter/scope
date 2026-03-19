using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Application.Interfaces;

/// <summary>
/// Defines the contract for user management operations.
/// Handles creation, updates, authentication support, and profile management.
/// </summary>
public interface IUserService
{
    /// <summary>
    /// Creates a new user account with the given details.
    /// </summary>
    Task<User> CreateUserAsync(string displayName, EmailAddress email, string password, UserRole role, CancellationToken cancellationToken = default);

    /// <summary>
    /// Retrieves a user by their unique identifier.
    /// </summary>
    Task<User?> GetUserAsync(Guid userId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Retrieves a user by their email address.
    /// </summary>
    Task<User?> GetUserByEmailAsync(string email, CancellationToken cancellationToken = default);

    /// <summary>
    /// Updates a user's profile information.
    /// </summary>
    Task<User> UpdateUserAsync(Guid userId, string displayName, PhoneNumber? phoneNumber, CancellationToken cancellationToken = default);

    /// <summary>
    /// Deactivates a user account.
    /// </summary>
    Task DeactivateUserAsync(Guid userId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Lists users with pagination support.
    /// </summary>
    Task<IReadOnlyList<User>> ListUsersAsync(int skip = 0, int take = 20, CancellationToken cancellationToken = default);
}
