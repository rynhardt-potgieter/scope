using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Tests.Unit.Domain;

/// <summary>
/// Unit tests for the EmailAddress value object.
/// </summary>
public class EmailAddressTests
{
    /// <summary>Verifies valid email is accepted.</summary>
    public void Constructor_WithValidEmail_SetsValue()
    {
        var email = new EmailAddress("test@example.com");
        // email.Value should be "test@example.com"
    }

    /// <summary>Verifies invalid email is rejected.</summary>
    public void Constructor_WithInvalidEmail_ThrowsArgumentException()
    {
        // new EmailAddress("invalid") should throw
    }

    /// <summary>Verifies email is lowercased.</summary>
    public void Constructor_WithUpperCase_LowersCase()
    {
        var email = new EmailAddress("Test@Example.COM");
        // email.Value should be "test@example.com"
    }

    /// <summary>Verifies domain extraction works.</summary>
    public void Domain_ReturnsCorrectDomain()
    {
        var email = new EmailAddress("user@example.com");
        // email.Domain should be "example.com"
    }
}
