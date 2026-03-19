import { Logger } from '../../shared/utils/Logger';
import { Notification } from '../models/Notification';

/** Queue entry with priority */
interface QueueEntry {
  notification: Notification;
  priority: number;
  addedAt: Date;
}

/** Priority queue for batching and ordering notification delivery */
export class NotificationQueue {
  private queue: QueueEntry[];
  private logger: Logger;
  private maxSize: number;

  constructor(maxSize: number = 10000) {
    this.queue = [];
    this.logger = new Logger('NotificationQueue');
    this.maxSize = maxSize;
  }

  /** Add a notification to the queue with a priority (lower = higher priority) */
  enqueue(notification: Notification, priority: number = 5): boolean {
    if (this.queue.length >= this.maxSize) {
      this.logger.warn('Queue is full', { maxSize: this.maxSize });
      return false;
    }
    this.queue.push({ notification, priority, addedAt: new Date() });
    this.queue.sort((a, b) => a.priority - b.priority);
    return true;
  }

  /** Remove and return the highest-priority notification */
  dequeue(): Notification | null {
    const entry = this.queue.shift();
    return entry?.notification ?? null;
  }

  /** Peek at the next notification without removing it */
  peek(): Notification | null {
    return this.queue[0]?.notification ?? null;
  }

  /** Get the current queue size */
  size(): number {
    return this.queue.length;
  }

  /** Check if the queue is empty */
  isEmpty(): boolean {
    return this.queue.length === 0;
  }

  /** Drain up to N items from the queue */
  drain(count: number): Notification[] {
    const items = this.queue.splice(0, count);
    return items.map((e) => e.notification);
  }
}
