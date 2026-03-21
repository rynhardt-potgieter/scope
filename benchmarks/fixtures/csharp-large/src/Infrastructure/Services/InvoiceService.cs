using CSharpLargeApi.Application.Interfaces;
using CSharpLargeApi.Domain.Entities;
using CSharpLargeApi.Domain.Exceptions;
using CSharpLargeApi.Domain.Interfaces;
using CSharpLargeApi.Domain.ValueObjects;

namespace CSharpLargeApi.Infrastructure.Services;

/// <summary>
/// Service responsible for invoice creation, management, and settlement.
/// Coordinates between the invoice repository and payment service
/// to handle invoice lifecycle operations.
/// </summary>
public class InvoiceService
{
    private readonly IRepository<Invoice> _invoiceRepository;
    private readonly IPaymentService _paymentService;
    private readonly INotificationService _notificationService;
    private int _invoiceSequence;

    /// <summary>
    /// Initializes the invoice service with required dependencies.
    /// </summary>
    public InvoiceService(
        IRepository<Invoice> invoiceRepository,
        IPaymentService paymentService,
        INotificationService notificationService)
    {
        _invoiceRepository = invoiceRepository ?? throw new ArgumentNullException(nameof(invoiceRepository));
        _paymentService = paymentService ?? throw new ArgumentNullException(nameof(paymentService));
        _notificationService = notificationService ?? throw new ArgumentNullException(nameof(notificationService));
        _invoiceSequence = 0;
    }

    /// <summary>
    /// Creates a new invoice for the specified user with the given line items.
    /// </summary>
    public async Task<Invoice> CreateInvoice(Guid userId, string currency, List<(string description, decimal amount, int quantity)> lineItems, CancellationToken cancellationToken = default)
    {
        var invoiceNumber = GenerateInvoiceNumber();
        var dueDate = DateTime.UtcNow.AddDays(30);

        var invoice = Invoice.Create(userId, invoiceNumber, dueDate, currency);

        foreach (var (description, amount, quantity) in lineItems)
        {
            var money = new Money(amount, currency);
            invoice.AddLineItem(description, money, quantity);
        }

        await _invoiceRepository.AddAsync(invoice, cancellationToken);

        await _notificationService.SendEmailAsync(
            userId,
            "New Invoice",
            $"Invoice {invoiceNumber} for {invoice.TotalAmount} has been issued. Due date: {dueDate:yyyy-MM-dd}",
            cancellationToken);

        return invoice;
    }

    /// <summary>
    /// Settles an invoice by processing a payment for the outstanding balance.
    /// Caller #6: This method calls IPaymentService.ProcessPayment to collect
    /// the remaining balance on the invoice.
    /// </summary>
    public async Task<Invoice> SettleInvoice(Guid invoiceId, string paymentMethodToken, CancellationToken cancellationToken = default)
    {
        var invoice = await _invoiceRepository.GetByIdAsync(invoiceId, cancellationToken);
        if (invoice is null)
        {
            throw new EntityNotFoundException("Invoice", invoiceId);
        }

        if (invoice.IsSettled)
        {
            throw new BusinessRuleException(
                "InvoiceAlreadySettled",
                $"Invoice {invoice.InvoiceNumber} is already settled.",
                "Invoice");
        }

        var remainingBalance = invoice.TotalAmount.Amount - invoice.PaidAmount.Amount;
        if (remainingBalance <= 0)
        {
            throw new BusinessRuleException(
                "NoBalanceDue",
                $"Invoice {invoice.InvoiceNumber} has no remaining balance.",
                "Invoice");
        }

        var paymentAmount = new Money(remainingBalance, invoice.TotalAmount.Currency);

        // Caller #6: InvoiceService.SettleInvoice calls ProcessPayment
        var payment = await _paymentService.ProcessPayment(
            invoice.UserId,
            paymentAmount,
            paymentMethodToken,
            cancellationToken);

        invoice.RecordPayment(paymentAmount);

        if (payment.Status == Domain.Enums.PaymentStatus.Completed)
        {
            await _invoiceRepository.SaveChangesAsync(cancellationToken);

            await _notificationService.SendEmailAsync(
                invoice.UserId,
                "Invoice Settled",
                $"Invoice {invoice.InvoiceNumber} has been fully settled. Thank you for your payment.",
                cancellationToken);
        }
        else
        {
            throw new BusinessRuleException(
                "InvoicePaymentFailed",
                $"Payment for invoice {invoice.InvoiceNumber} failed: {payment.FailureReason}",
                "Invoice");
        }

        return invoice;
    }

    /// <summary>
    /// Retrieves an invoice by its unique identifier.
    /// </summary>
    public async Task<Invoice?> GetInvoiceAsync(Guid invoiceId, CancellationToken cancellationToken = default)
    {
        return await _invoiceRepository.GetByIdAsync(invoiceId, cancellationToken);
    }

    /// <summary>
    /// Retrieves all invoices for a specific user.
    /// </summary>
    public async Task<IReadOnlyList<Invoice>> GetInvoicesByUserAsync(Guid userId, CancellationToken cancellationToken = default)
    {
        var all = await _invoiceRepository.GetAllAsync(cancellationToken);
        return all.Where(i => i.UserId == userId)
            .OrderByDescending(i => i.CreatedAt)
            .ToList()
            .AsReadOnly();
    }

    private string GenerateInvoiceNumber()
    {
        _invoiceSequence++;
        return $"INV-{DateTime.UtcNow:yyyy}-{_invoiceSequence:D5}";
    }
}
