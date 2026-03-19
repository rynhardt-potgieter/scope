using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents an authenticated user session.
/// Tracks session tokens, expiry, and device information for security auditing.
/// </summary>
public class Session : IEntity
{
    /// <summary>
    /// Gets the unique identifier for this session.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this session was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this session was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the ID of the user who owns this session.
    /// </summary>
    public Guid UserId { get; private set; }

    /// <summary>
    /// Gets the opaque refresh token for this session.
    /// </summary>
    public string RefreshToken { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the IP address from which this session was initiated.
    /// </summary>
    public string IpAddress { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the user agent string of the client device.
    /// </summary>
    public string UserAgent { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the timestamp when this session expires.
    /// </summary>
    public DateTime ExpiresAt { get; private set; }

    /// <summary>
    /// Gets whether this session has been revoked.
    /// </summary>
    public bool IsRevoked { get; private set; }

    /// <summary>
    /// Gets the timestamp of the last activity on this session.
    /// </summary>
    public DateTime LastActivityAt { get; private set; }

    /// <summary>
    /// Creates a new session for the specified user.
    /// </summary>
    public static Session Create(Guid userId, string refreshToken, string ipAddress, string userAgent, TimeSpan duration)
    {
        var now = DateTime.UtcNow;
        return new Session
        {
            Id = Guid.NewGuid(),
            UserId = userId,
            RefreshToken = refreshToken,
            IpAddress = ipAddress,
            UserAgent = userAgent,
            ExpiresAt = now.Add(duration),
            IsRevoked = false,
            LastActivityAt = now,
            CreatedAt = now
        };
    }

    /// <summary>
    /// Records activity on this session, extending the last-active timestamp.
    /// </summary>
    public void RecordActivity()
    {
        LastActivityAt = DateTime.UtcNow;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Revokes this session, preventing further use of its tokens.
    /// </summary>
    public void Revoke()
    {
        IsRevoked = true;
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Checks whether this session is currently valid and not expired.
    /// </summary>
    public bool IsValid()
    {
        return !IsRevoked && ExpiresAt > DateTime.UtcNow;
    }
}
