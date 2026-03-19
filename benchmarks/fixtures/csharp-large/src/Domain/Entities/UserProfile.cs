using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents extended profile information for a user.
/// Contains optional biographical data, preferences, and social links.
/// </summary>
public class UserProfile : IEntity
{
    /// <summary>
    /// Gets the unique identifier for this profile.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this profile was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this profile was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the ID of the user this profile belongs to.
    /// </summary>
    public Guid UserId { get; private set; }

    /// <summary>
    /// Gets or sets the user's biography text.
    /// </summary>
    public string? Bio { get; private set; }

    /// <summary>
    /// Gets or sets the URL to the user's avatar image.
    /// </summary>
    public string? AvatarUrl { get; private set; }

    /// <summary>
    /// Gets or sets the user's preferred timezone identifier.
    /// </summary>
    public string Timezone { get; private set; } = "UTC";

    /// <summary>
    /// Gets or sets the user's preferred locale (e.g. "en-US").
    /// </summary>
    public string Locale { get; private set; } = "en-US";

    /// <summary>
    /// Gets or sets the user's website URL.
    /// </summary>
    public string? WebsiteUrl { get; private set; }

    /// <summary>
    /// Gets or sets the user's location description.
    /// </summary>
    public string? Location { get; private set; }

    /// <summary>
    /// Creates a new user profile for the specified user.
    /// </summary>
    public static UserProfile Create(Guid userId)
    {
        return new UserProfile
        {
            Id = Guid.NewGuid(),
            UserId = userId,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Updates the biographical details of this profile.
    /// </summary>
    public void UpdateBio(string? bio, string? avatarUrl, string? location)
    {
        Bio = bio;
        AvatarUrl = avatarUrl;
        Location = location;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Updates the user's locale and timezone preferences.
    /// </summary>
    public void UpdatePreferences(string timezone, string locale)
    {
        Timezone = timezone;
        Locale = locale;
        UpdatedAt = DateTime.UtcNow;
    }
}
