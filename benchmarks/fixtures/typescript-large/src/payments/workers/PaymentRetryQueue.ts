import { Logger } from '../../shared/utils/Logger';
import { EntityId } from '../../types/common';
import { MAX_RETRY_ATTEMPTS, RETRY_DELAY_BASE_MS } from '../../types/constants';

/** A queued retry job */
export interface RetryJob {
  paymentId: EntityId;
  attempt: number;
  scheduledAt: Date;
  reason: string;
}

/** Queue for managing failed payment retries with exponential backoff */
export class PaymentRetryQueue {
  private queue: RetryJob[];
  private logger: Logger;

  constructor() {
    this.queue = [];
    this.logger = new Logger('PaymentRetryQueue');
  }

  /** Add a failed payment to the retry queue */
  enqueue(paymentId: EntityId, reason: string, attempt: number = 1): boolean {
    if (attempt > MAX_RETRY_ATTEMPTS) {
      this.logger.warn('Max retry attempts exceeded', { paymentId, attempt });
      return false;
    }

    const delay = RETRY_DELAY_BASE_MS * Math.pow(2, attempt - 1);
    const scheduledAt = new Date(Date.now() + delay);

    const job: RetryJob = { paymentId, attempt, scheduledAt, reason };
    this.queue.push(job);
    this.logger.info('Payment queued for retry', { paymentId, attempt, scheduledAt: scheduledAt.toISOString() });
    return true;
  }

  /** Dequeue the next job that is ready for processing */
  dequeue(): RetryJob | null {
    const now = Date.now();
    const readyIndex = this.queue.findIndex((job) => job.scheduledAt.getTime() <= now);
    if (readyIndex === -1) return null;
    const [job] = this.queue.splice(readyIndex, 1);
    this.logger.debug('Payment dequeued for retry', { paymentId: job.paymentId, attempt: job.attempt });
    return job;
  }

  /** Get the current queue size */
  size(): number {
    return this.queue.length;
  }

  /** Check if a payment is already in the retry queue */
  isQueued(paymentId: EntityId): boolean {
    return this.queue.some((job) => job.paymentId === paymentId);
  }
}
