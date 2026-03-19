using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Domain.Entities;

namespace CSharpLargeApi.Application.Mappings;

/// <summary>
/// Provides mapping methods between User entities and UserDto objects.
/// Centralizes the mapping logic to avoid duplication across handlers.
/// </summary>
public static class UserMappingProfile
{
    /// <summary>
    /// Maps a User entity to a UserDto.
    /// Excludes sensitive fields like PasswordHash.
    /// </summary>
    public static UserDto ToDto(User user)
    {
        return new UserDto
        {
            Id = user.Id,
            DisplayName = user.DisplayName,
            Email = user.Email.Value,
            Role = user.Role.ToString(),
            IsActive = user.IsActive,
            PhoneNumber = user.PhoneNumber?.Value,
            CreatedAt = user.CreatedAt
        };
    }

    /// <summary>
    /// Maps a collection of User entities to UserDto objects.
    /// </summary>
    public static IReadOnlyList<UserDto> ToDtoList(IEnumerable<User> users)
    {
        return users.Select(ToDto).ToList().AsReadOnly();
    }
}
