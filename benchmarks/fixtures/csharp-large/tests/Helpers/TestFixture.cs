using CSharpLargeApi.Infrastructure.External;
using CSharpLargeApi.Infrastructure.Persistence;

namespace CSharpLargeApi.Tests.Helpers;

/// <summary>
/// Test fixture that provides a configured set of services for integration tests.
/// Creates in-memory implementations of external dependencies.
/// </summary>
public class TestFixture : IDisposable
{
    /// <summary>
    /// Gets the in-memory database context.
    /// </summary>
    public AppDbContext DbContext { get; }

    /// <summary>
    /// Gets the test Stripe client.
    /// </summary>
    public StripeClient StripeClient { get; }

    /// <summary>
    /// Gets the test SendGrid client.
    /// </summary>
    public SendGridClient SendGridClient { get; }

    /// <summary>
    /// Gets the test Redis client.
    /// </summary>
    public RedisClient RedisClient { get; }

    /// <summary>
    /// Creates a new test fixture with in-memory dependencies.
    /// </summary>
    public TestFixture()
    {
        DbContext = new AppDbContext();
        StripeClient = new StripeClient("sk_test_fixture");
        SendGridClient = new SendGridClient("sg_test_fixture");
        RedisClient = new RedisClient("localhost:6379");
    }

    /// <summary>
    /// Generates a random user ID for testing.
    /// </summary>
    public Guid NewUserId() => Guid.NewGuid();

    /// <summary>
    /// Generates a random payment method token for testing.
    /// </summary>
    public string NewPaymentToken() => $"pm_test_{Guid.NewGuid():N}";

    /// <summary>
    /// Disposes the test fixture and its resources.
    /// </summary>
    public void Dispose()
    {
        // Cleanup resources
        GC.SuppressFinalize(this);
    }
}
