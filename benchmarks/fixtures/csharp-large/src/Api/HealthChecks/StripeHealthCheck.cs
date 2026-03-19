using CSharpLargeApi.Infrastructure.External;

namespace CSharpLargeApi.Api.HealthChecks;

/// <summary>
/// Health check that verifies Stripe gateway connectivity.
/// </summary>
public class StripeHealthCheck
{
    private readonly StripeClient _stripeClient;

    /// <summary>
    /// Initializes the health check with the Stripe client.
    /// </summary>
    public StripeHealthCheck(StripeClient stripeClient)
    {
        _stripeClient = stripeClient ?? throw new ArgumentNullException(nameof(stripeClient));
    }

    /// <summary>
    /// Checks whether Stripe is reachable by validating a test token.
    /// </summary>
    public async Task<HealthCheckResult> CheckAsync(CancellationToken cancellationToken = default)
    {
        try
        {
            var isValid = await _stripeClient.ValidatePaymentMethodAsync(
                "pm_health_check_test_token_12345678", cancellationToken);

            return new HealthCheckResult(true, "Stripe gateway is reachable.");
        }
        catch (Exception ex)
        {
            return new HealthCheckResult(false, $"Stripe check failed: {ex.Message}");
        }
    }
}
