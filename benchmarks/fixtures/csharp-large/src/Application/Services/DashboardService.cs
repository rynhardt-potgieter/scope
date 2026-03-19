using CSharpLargeApi.Application.Interfaces;

namespace CSharpLargeApi.Application.Services;

/// <summary>
/// Application service that aggregates data for admin dashboard views.
/// Provides summary statistics across multiple domains.
/// </summary>
public class DashboardService
{
    private readonly IPaymentService _paymentService;
    private readonly IUserService _userService;
    private readonly INotificationService _notificationService;
    private readonly ICacheService _cacheService;

    /// <summary>
    /// Initializes the service with required dependencies.
    /// </summary>
    public DashboardService(
        IPaymentService paymentService,
        IUserService userService,
        INotificationService notificationService,
        ICacheService cacheService)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _userService = userService ?? throw new ArgumentNullException(nameof(userService));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
        _cacheService = cacheService ?? throw new ArgumentNullException(nameof(cacheService));
    }

    /// <summary>
    /// Retrieves summary statistics for the admin dashboard.
    /// </summary>
    public async Task<DashboardSummary> GetSummaryAsync(CancellationToken cancellationToken = default)
    {
        var cacheKey = "dashboard:summary";
        var cached = await _cacheService.GetAsync<DashboardSummary>(cacheKey, cancellationToken);
        if (cached is not null)
        {
            return cached;
        }

        var users = await _userService.ListUsersAsync(0, 1, cancellationToken);
        var summary = new DashboardSummary
        {
            TotalUsers = users.Count,
            GeneratedAt = DateTime.UtcNow
        };

        await _cacheService.SetAsync(cacheKey, summary, TimeSpan.FromMinutes(5), cancellationToken);

        return summary;
    }
}

/// <summary>
/// Summary data for the admin dashboard.
/// </summary>
public class DashboardSummary
{
    /// <summary>Gets or sets the total user count.</summary>
    public int TotalUsers { get; set; }

    /// <summary>Gets or sets the total payment count.</summary>
    public int TotalPayments { get; set; }

    /// <summary>Gets or sets the total revenue amount.</summary>
    public decimal TotalRevenue { get; set; }

    /// <summary>Gets or sets the active subscription count.</summary>
    public int ActiveSubscriptions { get; set; }

    /// <summary>Gets or sets the pending notification count.</summary>
    public int PendingNotifications { get; set; }

    /// <summary>Gets or sets when this summary was generated.</summary>
    public DateTime GeneratedAt { get; set; }
}
