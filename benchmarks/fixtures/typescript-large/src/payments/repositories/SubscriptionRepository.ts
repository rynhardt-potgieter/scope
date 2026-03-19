import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Subscription } from '../models/Subscription';
import { EntityId } from '../../types/common';
import { SubscriptionStatus } from '../../types/enums';

/** Repository for subscription persistence */
export class SubscriptionRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('SubscriptionRepository');
  }

  /** Persist a new subscription */
  async save(subscription: Subscription): Promise<Subscription> {
    await this.db.execute(
      `INSERT INTO subscriptions (id, user_id, plan_name, status, billing_interval, amount, currency, current_period_start, current_period_end, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)`,
      [subscription.id, subscription.userId, subscription.planName, subscription.status, subscription.billingInterval, subscription.amount.amount, subscription.amount.currency, subscription.currentPeriodStart, subscription.currentPeriodEnd, subscription.createdAt],
    );
    this.logger.debug('Subscription saved', { subscriptionId: subscription.id });
    return subscription;
  }

  /** Find a subscription by ID */
  async findById(subscriptionId: EntityId): Promise<Subscription | null> {
    const result = await this.db.query<Subscription>('SELECT * FROM subscriptions WHERE id = $1', [subscriptionId]);
    return result.rows[0] ?? null;
  }

  /** Find all active subscriptions */
  async findActive(): Promise<Subscription[]> {
    const result = await this.db.query<Subscription>(
      'SELECT * FROM subscriptions WHERE status = $1',
      [SubscriptionStatus.ACTIVE],
    );
    return result.rows;
  }

  /** Update the billing period after renewal */
  async updatePeriod(subscriptionId: EntityId, periodStart: Date, periodEnd: Date): Promise<Subscription | null> {
    await this.db.execute(
      'UPDATE subscriptions SET current_period_start = $1, current_period_end = $2, next_billing_date = $3, failed_payment_attempts = 0, updated_at = $4 WHERE id = $5',
      [periodStart, periodEnd, periodEnd, new Date(), subscriptionId],
    );
    return this.findById(subscriptionId);
  }

  /** Increment the failed payment attempts counter */
  async incrementFailedAttempts(subscriptionId: EntityId): Promise<void> {
    await this.db.execute(
      'UPDATE subscriptions SET failed_payment_attempts = failed_payment_attempts + 1, status = $1, updated_at = $2 WHERE id = $3',
      [SubscriptionStatus.PAST_DUE, new Date(), subscriptionId],
    );
  }

  /** Cancel a subscription */
  async cancel(subscriptionId: EntityId, reason: string): Promise<Subscription | null> {
    await this.db.execute(
      'UPDATE subscriptions SET status = $1, cancelled_at = $2, cancel_reason = $3, updated_at = $4 WHERE id = $5',
      [SubscriptionStatus.CANCELLED, new Date(), reason, new Date(), subscriptionId],
    );
    return this.findById(subscriptionId);
  }
}
