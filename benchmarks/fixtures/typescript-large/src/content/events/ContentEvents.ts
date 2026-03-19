import { DomainEvent, EventType, EventMetadata } from '../../types/events';
import { EntityId } from '../../types/common';

/** Create a ContentCreated event */
export function createContentCreatedEvent(contentId: EntityId, authorId: EntityId, metadata: EventMetadata): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.CONTENT_CREATED,
    timestamp: new Date(),
    aggregateId: contentId,
    aggregateType: 'Content',
    payload: { contentId, authorId },
    metadata,
  };
}

/** Create a ContentPublished event */
export function createContentPublishedEvent(contentId: EntityId, metadata: EventMetadata): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.CONTENT_PUBLISHED,
    timestamp: new Date(),
    aggregateId: contentId,
    aggregateType: 'Content',
    payload: { contentId },
    metadata,
  };
}

/** Create a ContentUpdated event */
export function createContentUpdatedEvent(contentId: EntityId, fields: string[], metadata: EventMetadata): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.CONTENT_UPDATED,
    timestamp: new Date(),
    aggregateId: contentId,
    aggregateType: 'Content',
    payload: { contentId, updatedFields: fields },
    metadata,
  };
}
