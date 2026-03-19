import { Logger } from './Logger';
import { DomainEvent, EventHandler, EventType } from '../../types/events';

/** Simple in-process event bus for domain events */
export class EventBus {
  private handlers: Map<EventType, EventHandler[]>;
  private logger: Logger;

  constructor() {
    this.handlers = new Map();
    this.logger = new Logger('EventBus');
  }

  /** Subscribe a handler to a specific event type */
  on(eventType: EventType, handler: EventHandler): void {
    const existing = this.handlers.get(eventType) ?? [];
    existing.push(handler);
    this.handlers.set(eventType, existing);
    this.logger.debug('Handler registered', { eventType, handlerCount: existing.length });
  }

  /** Publish an event to all registered handlers */
  async emit(event: DomainEvent): Promise<void> {
    const handlers = this.handlers.get(event.type) ?? [];
    this.logger.debug('Emitting event', { eventType: event.type, handlerCount: handlers.length });
    for (const handler of handlers) {
      try {
        await handler(event);
      } catch (error) {
        this.logger.error('Event handler failed', { eventType: event.type, error: String(error) });
      }
    }
  }

  /** Remove all handlers for a specific event type */
  off(eventType: EventType): void {
    this.handlers.delete(eventType);
  }

  /** Remove all handlers */
  clear(): void {
    this.handlers.clear();
  }
}
