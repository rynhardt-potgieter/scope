using CSharpApi.Payments;

namespace CSharpApi.Workers;

/// <summary>
/// Background worker that retries failed payments on a schedule.
/// Implements a simple retry loop with configurable maximum attempts.
/// </summary>
public class PaymentRetryWorker
{
    private readonly IPaymentService _paymentService;
    private readonly int _maxRetries;

    /// <summary>
    /// Initializes a new PaymentRetryWorker with the given service and retry limit.
    /// </summary>
    public PaymentRetryWorker(IPaymentService paymentService, int maxRetries = 3)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _maxRetries = maxRetries;
    }

    /// <summary>
    /// Run a retry attempt for a failed payment.
    /// Tries up to maxRetries times, returning the result on first success
    /// or null if all attempts fail.
    /// </summary>
    public PaymentResult? Run(string userId, decimal amount, CardDetails card)
    {
        for (var attempt = 0; attempt < _maxRetries; attempt++)
        {
            try
            {
                // Call ProcessPayment — caller #6
                var result = _paymentService.ProcessPayment(amount, userId, card);
                if (result.Status == "success")
                {
                    return result;
                }
            }
            catch
            {
                // Retry on next iteration
                continue;
            }
        }

        return null;
    }

    /// <summary>
    /// Process a batch of failed payment retries.
    /// Each item in the batch is tried independently.
    /// </summary>
    public List<PaymentResult> RunBatch(IEnumerable<(string UserId, decimal Amount, CardDetails Card)> failedPayments)
    {
        var results = new List<PaymentResult>();

        foreach (var (userId, amount, card) in failedPayments)
        {
            var result = Run(userId, amount, card);
            if (result != null)
            {
                results.Add(result);
            }
        }

        return results;
    }
}
