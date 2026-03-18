namespace CSharpApi.Users;

/// <summary>
/// Data access layer for user records.
/// </summary>
public interface IUserRepository
{
    /// <summary>
    /// Find a user by their unique ID.
    /// Returns null if no user is found.
    /// </summary>
    User? FindById(string id);

    /// <summary>
    /// Find a user by their email address.
    /// Returns null if no user is found with the given email.
    /// </summary>
    User? FindByEmail(string email);

    /// <summary>
    /// Create a new user record and return the created user.
    /// </summary>
    User Create(string email, string name);
}
