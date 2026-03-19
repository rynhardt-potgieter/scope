import { DomainEvent, EventType, EventMetadata } from '../../types/events';
import { EntityId } from '../../types/common';

/** Create a NotificationSent event */
export function createNotificationSentEvent(
  notificationId: EntityId,
  userId: EntityId,
  channel: string,
  metadata: EventMetadata,
): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.NOTIFICATION_SENT,
    timestamp: new Date(),
    aggregateId: notificationId,
    aggregateType: 'Notification',
    payload: { notificationId, userId, channel },
    metadata,
  };
}

/** Create a NotificationFailed event */
export function createNotificationFailedEvent(
  notificationId: EntityId,
  userId: EntityId,
  reason: string,
  metadata: EventMetadata,
): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.NOTIFICATION_FAILED,
    timestamp: new Date(),
    aggregateId: notificationId,
    aggregateType: 'Notification',
    payload: { notificationId, userId, reason },
    metadata,
  };
}
