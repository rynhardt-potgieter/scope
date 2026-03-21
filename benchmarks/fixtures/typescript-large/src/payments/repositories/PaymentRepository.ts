import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Payment } from '../models/Payment';
import { EntityId } from '../../types/common';
import { PaymentStatus } from '../../types/enums';

/** Repository for payment persistence */
export class PaymentRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('PaymentRepository');
  }

  /** Persist a new payment record */
  async save(payment: Payment): Promise<Payment> {
    await this.db.execute(
      `INSERT INTO payments (id, user_id, amount, currency, status, processor, description, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)`,
      [payment.id, payment.userId, payment.amount.amount, payment.amount.currency, payment.status, payment.processor, payment.description, payment.createdAt],
    );
    this.logger.debug('Payment saved', { paymentId: payment.id });
    return payment;
  }

  /** Find a payment by ID */
  async findById(paymentId: EntityId): Promise<Payment | null> {
    const result = await this.db.query<Payment>('SELECT * FROM payments WHERE id = $1', [paymentId]);
    return result.rows[0] ?? null;
  }

  /** Find payments for a specific user */
  async findByUser(userId: EntityId, limit: number = 20): Promise<Payment[]> {
    const result = await this.db.query<Payment>(
      'SELECT * FROM payments WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2',
      [userId, limit],
    );
    return result.rows;
  }

  /** Find payments within a date range */
  async findByDateRange(start: Date, end: Date): Promise<Payment[]> {
    try {
      const result = await this.db.query<Payment>(
        'SELECT * FROM payments WHERE created_at >= $1 AND created_at <= $2 ORDER BY created_at DESC',
        [start, end],
      );
      return result.rows;
    } catch (error) {
      this.logger.error('Failed to query payments by date range', { start, end });
      return [];
    }
  }

  /** Update the status of a payment */
  async updateStatus(
    paymentId: EntityId,
    status: PaymentStatus,
    processorTransactionId?: string | null,
    failureReason?: string | null,
  ): Promise<void> {
    await this.db.execute(
      'UPDATE payments SET status = $1, processor_transaction_id = $2, failure_reason = $3, updated_at = $4 WHERE id = $5',
      [status, processorTransactionId ?? null, failureReason ?? null, new Date(), paymentId],
    );
    this.logger.debug('Payment status updated', { paymentId, status });
  }
}
