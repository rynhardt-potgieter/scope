import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Refund, RefundStatus } from '../models/Refund';
import { EntityId } from '../../types/common';

/** Repository for refund persistence */
export class RefundRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('RefundRepository');
  }

  /** Save a new refund */
  async save(refund: Refund): Promise<Refund> {
    await this.db.execute(
      `INSERT INTO refunds (id, payment_id, user_id, amount, currency, reason, status, notes, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)`,
      [refund.id, refund.paymentId, refund.userId, refund.amount.amount, refund.amount.currency, refund.reason, refund.status, refund.notes, refund.createdAt],
    );
    this.logger.debug('Refund saved', { refundId: refund.id });
    return refund;
  }

  /** Find a refund by ID */
  async findById(refundId: EntityId): Promise<Refund | null> {
    const result = await this.db.query<Refund>('SELECT * FROM refunds WHERE id = $1', [refundId]);
    return result.rows[0] ?? null;
  }

  /** Find refunds for a payment */
  async findByPaymentId(paymentId: EntityId): Promise<Refund[]> {
    const result = await this.db.query<Refund>(
      'SELECT * FROM refunds WHERE payment_id = $1 ORDER BY created_at DESC',
      [paymentId],
    );
    return result.rows;
  }

  /** Find refunds for a user */
  async findByUserId(userId: EntityId, limit: number = 20): Promise<Refund[]> {
    const result = await this.db.query<Refund>(
      'SELECT * FROM refunds WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2',
      [userId, limit],
    );
    return result.rows;
  }

  /** Update refund status */
  async updateStatus(refundId: EntityId, status: RefundStatus): Promise<void> {
    await this.db.execute('UPDATE refunds SET status = $1, updated_at = $2 WHERE id = $3', [status, new Date(), refundId]);
  }
}
