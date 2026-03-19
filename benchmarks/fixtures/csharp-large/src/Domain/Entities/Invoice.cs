using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Domain.Entities;

/// <summary>
/// Represents a billing invoice issued to a customer.
/// An invoice may reference one or more payments and tracks
/// its own lifecycle from draft through settlement.
/// </summary>
public class Invoice : IAggregateRoot
{
    private readonly List<IDomainEvent> _domainEvents = new();
    private readonly List<InvoiceLineItem> _lineItems = new();

    /// <summary>
    /// Gets the unique identifier for this invoice.
    /// </summary>
    public Guid Id { get; private set; }

    /// <summary>
    /// Gets the timestamp when this invoice was created.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the timestamp when this invoice was last modified.
    /// </summary>
    public DateTime? UpdatedAt { get; private set; }

    /// <summary>
    /// Gets the human-readable invoice number (e.g. INV-2024-00042).
    /// </summary>
    public string InvoiceNumber { get; private set; } = string.Empty;

    /// <summary>
    /// Gets the ID of the user this invoice is billed to.
    /// </summary>
    public Guid UserId { get; private set; }

    /// <summary>
    /// Gets the total amount due on this invoice.
    /// </summary>
    public Money TotalAmount { get; private set; } = null!;

    /// <summary>
    /// Gets the amount that has been paid so far.
    /// </summary>
    public Money PaidAmount { get; private set; } = null!;

    /// <summary>
    /// Gets whether this invoice has been fully settled.
    /// </summary>
    public bool IsSettled { get; private set; }

    /// <summary>
    /// Gets the due date for this invoice.
    /// </summary>
    public DateTime DueDate { get; private set; }

    /// <summary>
    /// Gets the read-only collection of line items on this invoice.
    /// </summary>
    public IReadOnlyList<InvoiceLineItem> LineItems => _lineItems.AsReadOnly();

    /// <summary>
    /// Gets the collection of domain events raised by this aggregate.
    /// </summary>
    public IReadOnlyCollection<IDomainEvent> DomainEvents => _domainEvents.AsReadOnly();

    /// <summary>
    /// Creates a new invoice for the specified user.
    /// </summary>
    public static Invoice Create(Guid userId, string invoiceNumber, DateTime dueDate, string currency)
    {
        return new Invoice
        {
            Id = Guid.NewGuid(),
            UserId = userId,
            InvoiceNumber = invoiceNumber,
            DueDate = dueDate,
            TotalAmount = new Money(0m, currency),
            PaidAmount = new Money(0m, currency),
            IsSettled = false,
            CreatedAt = DateTime.UtcNow
        };
    }

    /// <summary>
    /// Adds a line item to this invoice and recalculates the total.
    /// </summary>
    public void AddLineItem(string description, Money amount, int quantity)
    {
        var lineItem = new InvoiceLineItem(description, amount, quantity);
        _lineItems.Add(lineItem);
        RecalculateTotal();
        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Records a payment against this invoice.
    /// Marks the invoice as settled if the full amount has been paid.
    /// </summary>
    public void RecordPayment(Money paymentAmount)
    {
        PaidAmount = new Money(PaidAmount.Amount + paymentAmount.Amount, PaidAmount.Currency);

        if (PaidAmount.Amount >= TotalAmount.Amount)
        {
            IsSettled = true;
        }

        UpdatedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Clears all pending domain events from this aggregate.
    /// </summary>
    public void ClearDomainEvents()
    {
        _domainEvents.Clear();
    }

    private void RecalculateTotal()
    {
        var total = _lineItems.Sum(li => li.Amount.Amount * li.Quantity);
        TotalAmount = new Money(total, TotalAmount.Currency);
    }
}

/// <summary>
/// Represents a single line item on an invoice.
/// </summary>
public class InvoiceLineItem
{
    /// <summary>
    /// Gets the description of this line item.
    /// </summary>
    public string Description { get; }

    /// <summary>
    /// Gets the unit price for this line item.
    /// </summary>
    public Money Amount { get; }

    /// <summary>
    /// Gets the quantity of units for this line item.
    /// </summary>
    public int Quantity { get; }

    /// <summary>
    /// Creates a new invoice line item.
    /// </summary>
    public InvoiceLineItem(string description, Money amount, int quantity)
    {
        Description = description;
        Amount = amount;
        Quantity = quantity;
    }
}
