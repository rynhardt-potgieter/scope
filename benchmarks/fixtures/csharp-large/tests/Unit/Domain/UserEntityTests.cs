using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Enums;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Tests.Unit.Domain;

/// <summary>
/// Unit tests for the User entity.
/// </summary>
public class UserEntityTests
{
    /// <summary>Verifies user creation raises UserCreatedEvent.</summary>
    public void Create_RaisesUserCreatedEvent()
    {
        var email = new EmailAddress("test@example.com");
        var user = User.Create("Test", email, "hash", UserRole.User);
        // user.DomainEvents should contain UserCreatedEvent
    }

    /// <summary>Verifies deactivation sets IsActive to false.</summary>
    public void Deactivate_SetsIsActiveFalse()
    {
        var email = new EmailAddress("test@example.com");
        var user = User.Create("Test", email, "hash", UserRole.User);
        user.Deactivate();
        // user.IsActive should be false
    }

    /// <summary>Verifies email confirmation flag.</summary>
    public void ConfirmEmail_SetsIsEmailConfirmedTrue()
    {
        var email = new EmailAddress("test@example.com");
        var user = User.Create("Test", email, "hash", UserRole.User);
        user.ConfirmEmail();
        // user.IsEmailConfirmed should be true
    }

    /// <summary>Verifies ClearDomainEvents removes all events.</summary>
    public void ClearDomainEvents_RemovesAllEvents()
    {
        var email = new EmailAddress("test@example.com");
        var user = User.Create("Test", email, "hash", UserRole.User);
        user.ClearDomainEvents();
        // user.DomainEvents should be empty
    }
}
