using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Exceptions;

namespace CSharpLargeApi.Application.Queries.GetUser;

/// <summary>
/// Handles the GetUserQuery by retrieving user data from the service
/// and mapping it to a DTO. Uses caching when available.
/// </summary>
public class GetUserHandler
{
    private readonly IUserService _userService;
    private readonly ICacheService _cacheService;

    /// <summary>
    /// Initializes the handler with required dependencies.
    /// </summary>
    public GetUserHandler(IUserService userService, ICacheService cacheService)
    {
        _userService = userService ?? throw new ArgumentNullException(nameof(userService));
        _cacheService = cacheService ?? throw new ArgumentNullException(nameof(cacheService));
    }

    /// <summary>
    /// Handles the query by checking the cache first, then falling back to the service.
    /// </summary>
    public async Task<UserDto> Handle(GetUserQuery query, CancellationToken cancellationToken)
    {
        var cacheKey = $"user:{query.UserId}";
        var cached = await _cacheService.GetAsync<UserDto>(cacheKey, cancellationToken);
        if (cached is not null)
        {
            return cached;
        }

        var user = await _userService.GetUserAsync(query.UserId, cancellationToken);
        if (user is null)
        {
            throw new EntityNotFoundException("User", query.UserId);
        }

        var dto = new UserDto
        {
            Id = user.Id,
            DisplayName = user.DisplayName,
            Email = user.Email.Value,
            Role = user.Role.ToString(),
            IsActive = user.IsActive,
            CreatedAt = user.CreatedAt
        };

        await _cacheService.SetAsync(cacheKey, dto, TimeSpan.FromMinutes(5), cancellationToken);

        return dto;
    }
}
