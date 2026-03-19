import { DomainEvent, EventType, EventMetadata } from '../../types/events';
import { EntityId } from '../../types/common';

/** Create a UserCreated event */
export function createUserCreatedEvent(userId: EntityId, email: string, metadata: EventMetadata): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.USER_CREATED,
    timestamp: new Date(),
    aggregateId: userId,
    aggregateType: 'User',
    payload: { userId, email },
    metadata,
  };
}

/** Create a UserUpdated event */
export function createUserUpdatedEvent(userId: EntityId, fields: string[], metadata: EventMetadata): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.USER_UPDATED,
    timestamp: new Date(),
    aggregateId: userId,
    aggregateType: 'User',
    payload: { userId, updatedFields: fields },
    metadata,
  };
}

/** Create a UserDeleted event */
export function createUserDeletedEvent(userId: EntityId, metadata: EventMetadata): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.USER_DELETED,
    timestamp: new Date(),
    aggregateId: userId,
    aggregateType: 'User',
    payload: { userId },
    metadata,
  };
}
