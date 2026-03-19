import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Invoice } from '../models/Invoice';
import { EntityId } from '../../types/common';
import { InvoiceStatus } from '../../types/enums';

/** Repository for invoice persistence */
export class InvoiceRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('InvoiceRepository');
  }

  /** Persist a new invoice */
  async save(invoice: Invoice): Promise<Invoice> {
    await this.db.execute(
      `INSERT INTO invoices (id, user_id, subscription_id, invoice_number, status, total, due_date, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`,
      [invoice.id, invoice.userId, invoice.subscriptionId, invoice.invoiceNumber, invoice.status, invoice.total.amount, invoice.dueDate, invoice.createdAt],
    );
    this.logger.debug('Invoice saved', { invoiceId: invoice.id });
    return invoice;
  }

  /** Find an invoice by ID */
  async findById(invoiceId: EntityId): Promise<Invoice | null> {
    const result = await this.db.query<Invoice>('SELECT * FROM invoices WHERE id = $1', [invoiceId]);
    return result.rows[0] ?? null;
  }

  /** Find invoices for a subscription */
  async findBySubscription(subscriptionId: EntityId): Promise<Invoice[]> {
    const result = await this.db.query<Invoice>(
      'SELECT * FROM invoices WHERE subscription_id = $1 ORDER BY created_at DESC',
      [subscriptionId],
    );
    return result.rows;
  }

  /** Update invoice status and optionally link to a payment */
  async updateStatus(invoiceId: EntityId, status: InvoiceStatus, paymentId?: EntityId): Promise<Invoice | null> {
    await this.db.execute(
      'UPDATE invoices SET status = $1, payment_id = $2, paid_at = $3, updated_at = $4 WHERE id = $5',
      [status, paymentId ?? null, status === InvoiceStatus.PAID ? new Date() : null, new Date(), invoiceId],
    );
    return this.findById(invoiceId);
  }
}
