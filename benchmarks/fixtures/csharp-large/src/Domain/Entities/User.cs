using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.Events;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a platform user with authentication credentials,
/// profile information, and role-based access control.
/// Serves as an aggregate root for user-related operations.
/// </summary>
public class User : IAggregateRoot
{
    private readonly List<IDomainEvent> _domainEvents = new();

    /// <summary>
    /// Gets the unique identifier for this user.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this user was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this user was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets or sets the user's display name.
    /// </summary>
    public string DisplayName { get; private set; } = string.Empty;

    /// <summary>
    /// Gets or sets the user's validated email address.
    /// </summary>
    public EmailAddress Email { get; private set; } = null!;

    /// <summary>
    /// Gets or sets the user's optional phone number.
    /// </summary>
    public PhoneNumber? PhoneNumber { get; private set; }

    /// <summary>
    /// Gets the user's assigned role within the platform.
    /// </summary>
    public UserRole Role { get; private set; }

    /// <summary>
    /// Gets whether the user's email has been confirmed.
    /// </summary>
    public bool IsEmailConfirmed { get; private set; }

    /// <summary>
    /// Gets whether the user account is currently active.
    /// </summary>
    public bool IsActive { get; private set; }

    /// <summary>
    /// Gets the hashed password for this user.
    /// Never store or log the plain-text password.
    /// </summary>
    public string PasswordHash { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the collection of domain events raised by this aggregate.
    /// </summary>
    public IReadOnlyCollection<IDomainEvent> DomainEvents => _domainEvents.AsReadOnly();

    /// <summary>
    /// Creates a new User instance with the required fields.
    /// Raises a UserCreatedEvent upon successful creation.
    /// </summary>
    public static User Create(string displayName, EmailAddress email, string passwordHash, UserRole role)
    {
        var user = new User
        {
            Id = Guid.NewGuid(),
            DisplayName = displayName,
            Email = email,
            PasswordHash = passwordHash,
            Role = role,
            IsActive = true,
            IsEmailConfirmed = false,
            CreatedAt = DateTime.UtcNow
        };

        user._domainEvents.Add(new UserCreatedEvent(user.Id, email.Value));
        return user;
    }

    /// <summary>
    /// Updates the user's profile information.
    /// </summary>
    public void UpdateProfile(string displayName, PhoneNumber? phoneNumber)
    {
        DisplayName = displayName;
        PhoneNumber = phoneNumber;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Confirms the user's email address after verification.
    /// </summary>
    public void ConfirmEmail()
    {
        IsEmailConfirmed = true;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Deactivates the user account, preventing further logins.
    /// </summary>
    public void Deactivate()
    {
        IsActive = false;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Reactivates a previously deactivated user account.
    /// </summary>
    public void Reactivate()
    {
        IsActive = true;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Clears all pending domain events from this aggregate.
    /// </summary>
    public void ClearDomainEvents()
    {
        _domainEvents.Clear();
    }
}
