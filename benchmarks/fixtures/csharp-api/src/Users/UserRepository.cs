namespace CSharpApi.Users;

/// <summary>
/// In-memory implementation of IUserRepository.
/// Simulates a database-backed user store for the benchmark fixture.
/// </summary>
public class UserRepository : IUserRepository
{
    private readonly Dictionary<string, User> _users = new();

    /// <summary>
    /// Find a user by their unique ID.
    /// Performs a direct dictionary lookup.
    /// </summary>
    public User? FindById(string id)
    {
        _users.TryGetValue(id, out var user);
        return user;
    }

    /// <summary>
    /// Find a user by their email address.
    /// Scans all users since email is not the primary key.
    /// </summary>
    public User? FindByEmail(string email)
    {
        foreach (var user in _users.Values)
        {
            if (string.Equals(user.Email, email, StringComparison.OrdinalIgnoreCase))
            {
                return user;
            }
        }

        return null;
    }

    /// <summary>
    /// Create a new user record and return the created user.
    /// Generates a unique ID based on the current timestamp.
    /// </summary>
    public User Create(string email, string name)
    {
        if (string.IsNullOrWhiteSpace(email))
        {
            throw new ArgumentException("Email is required", nameof(email));
        }

        if (string.IsNullOrWhiteSpace(name))
        {
            throw new ArgumentException("Name is required", nameof(name));
        }

        var user = new User
        {
            Id = $"user_{DateTime.UtcNow.Ticks}",
            Email = email,
            Name = name,
            CreatedAt = DateTime.UtcNow,
            IsActive = true
        };

        _users[user.Id] = user;
        return user;
    }
}
