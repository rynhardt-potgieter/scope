import { DatabaseClient } from '../../shared/database/DatabaseClient';
import { Logger } from '../../shared/utils/Logger';
import { Notification, DeliveryStatus } from '../models/Notification';
import { EntityId } from '../../types/common';

/** Repository for notification persistence */
export class NotificationRepository {
  private db: DatabaseClient;
  private logger: Logger;

  constructor(db: DatabaseClient) {
    this.db = db;
    this.logger = new Logger('NotificationRepository');
  }

  /** Save a new notification */
  async save(notification: Notification): Promise<Notification> {
    await this.db.execute(
      `INSERT INTO notifications (id, user_id, channel, subject, body, status, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7)`,
      [notification.id, notification.userId, notification.channel, notification.subject, notification.body, notification.status, notification.createdAt],
    );
    return notification;
  }

  /** Find notifications for a user */
  async findByUser(userId: EntityId, limit: number = 20): Promise<Notification[]> {
    const result = await this.db.query<Notification>(
      'SELECT * FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2',
      [userId, limit],
    );
    return result.rows;
  }

  /** Update notification delivery status */
  async updateStatus(notificationId: EntityId, status: DeliveryStatus, failureReason?: string): Promise<void> {
    const sentAt = status === DeliveryStatus.SENT ? new Date() : null;
    await this.db.execute(
      'UPDATE notifications SET status = $1, sent_at = $2, failure_reason = $3, updated_at = $4 WHERE id = $5',
      [status, sentAt, failureReason ?? null, new Date(), notificationId],
    );
  }

  /** Mark a notification as read */
  async markRead(notificationId: EntityId): Promise<void> {
    await this.db.execute('UPDATE notifications SET read_at = $1 WHERE id = $2', [new Date(), notificationId]);
  }

  /** Count unread notifications for a user */
  async countUnread(userId: EntityId): Promise<number> {
    const result = await this.db.query<{ count: number }>(
      'SELECT COUNT(*) as count FROM notifications WHERE user_id = $1 AND read_at IS NULL',
      [userId],
    );
    return result.rows[0]?.count ?? 0;
  }
}
