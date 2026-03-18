namespace CSharpApi.Users;

/// <summary>
/// Business logic layer for user operations.
/// Wraps the repository with validation and business rules.
/// </summary>
public class UserService
{
    private readonly IUserRepository _repository;

    /// <summary>
    /// Initializes a new UserService with the given repository.
    /// </summary>
    public UserService(IUserRepository repository)
    {
        _repository = repository ?? throw new ArgumentNullException(nameof(repository));
    }

    /// <summary>
    /// Get a user by ID. Throws if user not found.
    /// This is the primary lookup method used by controllers.
    /// </summary>
    public User GetUser(string userId)
    {
        // Caller #1 of UserRepository.FindById
        var user = _repository.FindById(userId);
        if (user == null)
        {
            throw new InvalidOperationException($"User {userId} not found");
        }

        return user;
    }

    /// <summary>
    /// Create a new user with the given email and name.
    /// Checks for duplicate emails before creating.
    /// </summary>
    public User CreateUser(string email, string name)
    {
        var existing = _repository.FindByEmail(email);
        if (existing != null)
        {
            throw new InvalidOperationException($"User with email {email} already exists");
        }

        return _repository.Create(email, name);
    }

    /// <summary>
    /// Check whether a user exists by their ID.
    /// </summary>
    public bool UserExists(string userId)
    {
        // Caller #2 of UserRepository.FindById
        var user = _repository.FindById(userId);
        return user != null;
    }
}
