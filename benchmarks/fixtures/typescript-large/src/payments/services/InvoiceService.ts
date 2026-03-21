import { Logger } from '../../shared/utils/Logger';
import { InvoiceRepository } from '../repositories/InvoiceRepository';
import { PaymentService } from './PaymentService';
import { Invoice, InvoiceLineItem } from '../models/Invoice';
import { EntityId } from '../../types/common';
import { InvoiceStatus, PaymentProcessor } from '../../types/enums';
import { Money, createMoney, addMoney, Currency } from '../../types/money';
import { NotFoundError, ValidationError } from '../../types/errors';
import { CryptoService } from '../../shared/utils/Crypto';

/** Service for creating, settling, and managing invoices */
export class InvoiceService {
  private invoiceRepo: InvoiceRepository;
  private paymentService: PaymentService;
  private logger: Logger;

  constructor(invoiceRepo: InvoiceRepository, paymentService: PaymentService) {
    this.invoiceRepo = invoiceRepo;
    this.paymentService = paymentService;
    this.logger = new Logger('InvoiceService');
  }

  /** Create a new invoice for a user */
  async createInvoice(
    userId: EntityId,
    lineItems: InvoiceLineItem[],
    dueDate: Date,
    notes: string = '',
  ): Promise<Invoice> {
    if (lineItems.length === 0) {
      throw new ValidationError('Invoice must have at least one line item');
    }

    const subtotal = lineItems.reduce(
      (sum, item) => addMoney(sum, item.totalPrice),
      createMoney(0, lineItems[0].unitPrice.currency),
    );
    const taxRate = 0.15;
    const tax = createMoney(subtotal.amount * taxRate, subtotal.currency);
    const total = addMoney(subtotal, tax);

    const crypto = new CryptoService();
    const invoice: Invoice = {
      id: `inv_${Date.now()}`,
      userId,
      subscriptionId: null,
      paymentId: null,
      invoiceNumber: `INV-${crypto.generateToken(8).toUpperCase()}`,
      status: InvoiceStatus.ISSUED,
      lineItems,
      subtotal,
      tax,
      total,
      dueDate,
      paidAt: null,
      notes,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    const saved = await this.invoiceRepo.save(invoice);
    this.logger.info('Invoice created', { invoiceId: saved.id, total: total.amount });
    return saved;
  }

  /** Settle an invoice by processing payment for the total amount */
  async settleInvoice(invoiceId: EntityId): Promise<Invoice> {
    const invoice = await this.invoiceRepo.findById(invoiceId);
    if (!invoice) {
      throw new NotFoundError('Invoice', invoiceId);
    }
    if (invoice.status !== InvoiceStatus.ISSUED && invoice.status !== InvoiceStatus.OVERDUE) {
      throw new ValidationError(`Cannot settle invoice with status ${invoice.status}`);
    }

    this.logger.info('Settling invoice', { invoiceId, total: invoice.total.amount });

    let paymentResult;
    try {
      paymentResult = await this.paymentService.processPayment(
        invoice.userId,
        invoice.total,
        PaymentProcessor.STRIPE,
        `Invoice ${invoice.invoiceNumber}`,
        `invoice_settle_${invoiceId}`,
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown payment error';
      this.logger.error('Invoice settlement payment failed', { invoiceId, error: message });
      throw error;
    }

    if (paymentResult.success) {
      const updated = await this.invoiceRepo.updateStatus(invoiceId, InvoiceStatus.PAID, paymentResult.paymentId);
      this.logger.info('Invoice settled', { invoiceId, paymentId: paymentResult.paymentId });
      return updated!;
    } else {
      this.logger.warn('Invoice settlement failed', { invoiceId, reason: paymentResult.failureReason });
      throw new ValidationError(`Payment failed: ${paymentResult.failureReason}`);
    }
  }

  /** Void an unpaid invoice */
  async voidInvoice(invoiceId: EntityId): Promise<Invoice> {
    const invoice = await this.invoiceRepo.findById(invoiceId);
    if (!invoice) {
      throw new NotFoundError('Invoice', invoiceId);
    }
    if (invoice.status === InvoiceStatus.PAID) {
      throw new ValidationError('Cannot void a paid invoice');
    }

    const updated = await this.invoiceRepo.updateStatus(invoiceId, InvoiceStatus.VOIDED);
    this.logger.info('Invoice voided', { invoiceId });
    return updated!;
  }

  /** Get an invoice by ID */
  async getInvoice(invoiceId: EntityId): Promise<Invoice> {
    const invoice = await this.invoiceRepo.findById(invoiceId);
    if (!invoice) {
      throw new NotFoundError('Invoice', invoiceId);
    }
    return invoice;
  }
}
