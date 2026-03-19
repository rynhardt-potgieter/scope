using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;
using CSharpLargeApi.Shared.Helpers;

namespace CSharpLargeApi.Infrastructure.Services;

/// <summary>
/// Implementation of the user service handling user CRUD operations,
/// password hashing, and profile management.
/// </summary>
public class UserService : IUserService
{
    private readonly IRepository<User> _userRepository;

    /// <summary>
    /// Initializes the user service with the user repository.
    /// </summary>
    public UserService(IRepository<User> userRepository)
    {
        _userRepository = userRepository ?? throw new ArgumentNullException(nameof(userRepository));
    }

    /// <summary>
    /// Creates a new user account with the given details.
    /// Hashes the password before storing.
    /// </summary>
    public async Task<User> CreateUserAsync(string displayName, EmailAddress email, string password, UserRole role, CancellationToken cancellationToken = default)
    {
        Guard.AgainstNullOrEmpty(displayName, nameof(displayName));
        Guard.AgainstNullOrEmpty(password, nameof(password));

        var existing = await GetUserByEmailAsync(email.Value, cancellationToken);
        if (existing is not null)
        {
            throw new BusinessRuleException(
                "UniqueEmailRequired",
                $"A user with email '{email.Value}' already exists.");
        }

        var passwordHash = CryptoHelper.HashPassword(password);
        var user = User.Create(displayName, email, passwordHash, role);

        await _userRepository.AddAsync(user, cancellationToken);
        return user;
    }

    /// <summary>
    /// Retrieves a user by their unique identifier.
    /// </summary>
    public async Task<User?> GetUserAsync(Guid userId, CancellationToken cancellationToken = default)
    {
        return await _userRepository.GetByIdAsync(userId, cancellationToken);
    }

    /// <summary>
    /// Retrieves a user by their email address.
    /// </summary>
    public async Task<User?> GetUserByEmailAsync(string email, CancellationToken cancellationToken = default)
    {
        var all = await _userRepository.GetAllAsync(cancellationToken);
        return all.FirstOrDefault(u => u.Email.Value == email.ToLowerInvariant());
    }

    /// <summary>
    /// Updates a user's profile information.
    /// </summary>
    public async Task<User> UpdateUserAsync(Guid userId, string displayName, PhoneNumber? phoneNumber, CancellationToken cancellationToken = default)
    {
        var user = await _userRepository.GetByIdAsync(userId, cancellationToken);
        if (user is null)
        {
            throw new EntityNotFoundException("User", userId);
        }

        user.UpdateProfile(displayName, phoneNumber);
        await _userRepository.SaveChangesAsync(cancellationToken);
        return user;
    }

    /// <summary>
    /// Deactivates a user account.
    /// </summary>
    public async Task DeactivateUserAsync(Guid userId, CancellationToken cancellationToken = default)
    {
        var user = await _userRepository.GetByIdAsync(userId, cancellationToken);
        if (user is null)
        {
            throw new EntityNotFoundException("User", userId);
        }

        user.Deactivate();
        await _userRepository.SaveChangesAsync(cancellationToken);
    }

    /// <summary>
    /// Lists users with pagination support.
    /// </summary>
    public async Task<IReadOnlyList<User>> ListUsersAsync(int skip = 0, int take = 20, CancellationToken cancellationToken = default)
    {
        var all = await _userRepository.GetAllAsync(cancellationToken);
        return all.Skip(skip).Take(take).ToList().AsReadOnly();
    }
}
