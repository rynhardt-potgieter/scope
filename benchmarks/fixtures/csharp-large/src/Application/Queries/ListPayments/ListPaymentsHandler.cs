using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Application.Interfaces;

namespace CSharpLargeApi.Application.Queries.ListPayments;

/// <summary>
/// Handles the ListPaymentsQuery by retrieving paginated payment records
/// for a specific user.
/// </summary>
public class ListPaymentsHandler
{
    private readonly IPaymentService _paymentService;

    /// <summary>
    /// Initializes the handler with the payment service dependency.
    /// </summary>
    public ListPaymentsHandler(IPaymentService paymentService)
    {
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
    }

    /// <summary>
    /// Handles the query by fetching payments and mapping them to DTOs.
    /// </summary>
    public async Task<IReadOnlyList<PaymentDto>> Handle(ListPaymentsQuery query, CancellationToken cancellationToken)
    {
        var payments = await _paymentService.GetPaymentsByUserAsync(
            query.UserId, query.Skip, query.Take, cancellationToken);

        return payments.Select(p => new PaymentDto
        {
            Id = p.Id,
            UserId = p.UserId,
            Amount = p.Amount.Amount,
            Currency = p.Amount.Currency,
            Status = p.Status.ToString(),
            GatewayTransactionId = p.GatewayTransactionId,
            CreatedAt = p.CreatedAt
        }).ToList().AsReadOnly();
    }
}
