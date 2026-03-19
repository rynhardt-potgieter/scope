using CSharpLargeApi.Application.DTOs;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.Interfaces;

namespace CSharpLargeApi.Application.Queries.GetInvoice;

/// <summary>
/// Handles the GetInvoiceQuery by retrieving the invoice from the repository
/// and mapping it to a DTO including line items.
/// </summary>
public class GetInvoiceHandler
{
    private readonly IRepository<Invoice> _invoiceRepository;

    /// <summary>
    /// Initializes the handler with the invoice repository.
    /// </summary>
    public GetInvoiceHandler(IRepository<Invoice> invoiceRepository)
    {
        _invoiceRepository = invoiceRepository ?? throw new ArgumentNullException(nameof(invoiceRepository));
    }

    /// <summary>
    /// Handles the query by fetching the invoice and mapping it.
    /// </summary>
    public async Task<InvoiceDto> Handle(GetInvoiceQuery query, CancellationToken cancellationToken)
    {
        var invoice = await _invoiceRepository.GetByIdAsync(query.InvoiceId, cancellationToken);
        if (invoice is null)
        {
            throw new EntityNotFoundException("Invoice", query.InvoiceId);
        }

        return new InvoiceDto
        {
            Id = invoice.Id,
            InvoiceNumber = invoice.InvoiceNumber,
            UserId = invoice.UserId,
            TotalAmount = invoice.TotalAmount.Amount,
            PaidAmount = invoice.PaidAmount.Amount,
            Currency = invoice.TotalAmount.Currency,
            IsSettled = invoice.IsSettled,
            DueDate = invoice.DueDate,
            LineItemCount = invoice.LineItems.Count,
            CreatedAt = invoice.CreatedAt
        };
    }
}
