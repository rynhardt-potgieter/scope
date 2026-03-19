import { EntityId } from './common';

/** All domain event types in the system */
export enum EventType {
  USER_CREATED = 'user.created',
  USER_UPDATED = 'user.updated',
  USER_DELETED = 'user.deleted',
  USER_LOGGED_IN = 'user.logged_in',
  USER_REGISTERED = 'user.registered',
  PAYMENT_PROCESSED = 'payment.processed',
  PAYMENT_FAILED = 'payment.failed',
  PAYMENT_REFUNDED = 'payment.refunded',
  SUBSCRIPTION_CREATED = 'subscription.created',
  SUBSCRIPTION_RENEWED = 'subscription.renewed',
  SUBSCRIPTION_CANCELLED = 'subscription.cancelled',
  CONTENT_CREATED = 'content.created',
  CONTENT_UPDATED = 'content.updated',
  CONTENT_PUBLISHED = 'content.published',
  CONTENT_DELETED = 'content.deleted',
  NOTIFICATION_SENT = 'notification.sent',
  NOTIFICATION_FAILED = 'notification.failed',
  INVOICE_CREATED = 'invoice.created',
  INVOICE_SETTLED = 'invoice.settled',
  INVOICE_VOIDED = 'invoice.voided',
}

/** Base domain event that all events extend */
export interface DomainEvent {
  id: EntityId;
  type: EventType;
  timestamp: Date;
  aggregateId: EntityId;
  aggregateType: string;
  payload: Record<string, unknown>;
  metadata: EventMetadata;
}

/** Metadata attached to every event */
export interface EventMetadata {
  correlationId: string;
  causationId?: string;
  userId?: EntityId;
  source: string;
}

/** Event handler function signature */
export type EventHandler = (event: DomainEvent) => Promise<void>;
