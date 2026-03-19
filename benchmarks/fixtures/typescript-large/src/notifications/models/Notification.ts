import { BaseEntity, EntityId } from '../../types/common';
import { NotificationChannel } from '../../types/enums';

/** Delivery status of a notification */
export enum DeliveryStatus {
  PENDING = 'pending',
  SENT = 'sent',
  DELIVERED = 'delivered',
  FAILED = 'failed',
  BOUNCED = 'bounced',
}

/** A notification record */
export interface Notification extends BaseEntity {
  userId: EntityId;
  channel: NotificationChannel;
  subject: string;
  body: string;
  status: DeliveryStatus;
  sentAt: Date | null;
  deliveredAt: Date | null;
  failureReason: string | null;
  retryCount: number;
  metadata: Record<string, unknown>;
}
