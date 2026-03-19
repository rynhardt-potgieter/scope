import { DomainEvent, EventType, EventMetadata } from '../../types/events';
import { EntityId } from '../../types/common';

/** Create a UserLoggedIn event */
export function createUserLoggedInEvent(userId: EntityId, metadata: EventMetadata): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.USER_LOGGED_IN,
    timestamp: new Date(),
    aggregateId: userId,
    aggregateType: 'User',
    payload: { userId },
    metadata,
  };
}

/** Create a UserRegistered event */
export function createUserRegisteredEvent(
  userId: EntityId,
  email: string,
  metadata: EventMetadata,
): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.USER_REGISTERED,
    timestamp: new Date(),
    aggregateId: userId,
    aggregateType: 'User',
    payload: { userId, email },
    metadata,
  };
}
