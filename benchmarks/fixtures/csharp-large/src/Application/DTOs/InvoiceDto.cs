namespace CSharpLargeApi.Application.DTOs;

/// <summary>
/// Data transfer object for invoice summary information.
/// </summary>
public class InvoiceDto
{
    /// <summary>
    /// Gets or sets the invoice identifier.
    /// </summary>
    public Guid Id { get; set; }

    /// <summary>
    /// Gets or sets the human-readable invoice number.
    /// </summary>
    public string InvoiceNumber { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets the user identifier.
    /// </summary>
    public Guid UserId { get; set; }

    /// <summary>
    /// Gets or sets the total invoice amount.
    /// </summary>
    public decimal TotalAmount { get; set; }

    /// <summary>
    /// Gets or sets the amount paid so far.
    /// </summary>
    public decimal PaidAmount { get; set; }

    /// <summary>
    /// Gets or sets the currency code.
    /// </summary>
    public string Currency { get; set; } = string.Empty;

    /// <summary>
    /// Gets or sets whether the invoice is fully settled.
    /// </summary>
    public bool IsSettled { get; set; }

    /// <summary>
    /// Gets or sets the payment due date.
    /// </summary>
    public DateTime DueDate { get; set; }

    /// <summary>
    /// Gets or sets the number of line items.
    /// </summary>
    public int LineItemCount { get; set; }

    /// <summary>
    /// Gets or sets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; set; }
}
