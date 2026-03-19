namespace CSharpLargeApi.Application.Queries.GetInvoice;

/// <summary>
/// Query to retrieve a single invoice by its unique identifier.
/// </summary>
public sealed class GetInvoiceQuery
{
    /// <summary>
    /// Gets the ID of the invoice to retrieve.
    /// </summary>
    public Guid InvoiceId { get; }

    /// <summary>
    /// Creates a new GetInvoiceQuery.
    /// </summary>
    public GetInvoiceQuery(Guid invoiceId)
    {
        InvoiceId = invoiceId;
    }
}
