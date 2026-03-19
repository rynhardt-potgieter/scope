import { Logger } from '../../shared/utils/Logger';
import { NotificationQueue } from './NotificationQueue';
import { NotificationService } from '../services/NotificationService';

/** Background worker that drains the notification queue and delivers messages */
export class DeliveryWorker {
  private queue: NotificationQueue;
  private notificationService: NotificationService;
  private logger: Logger;
  private batchSize: number;
  private running: boolean;

  constructor(queue: NotificationQueue, notificationService: NotificationService, batchSize: number = 10) {
    this.queue = queue;
    this.notificationService = notificationService;
    this.logger = new Logger('DeliveryWorker');
    this.batchSize = batchSize;
    this.running = false;
  }

  /** Start the delivery worker */
  async start(): Promise<void> {
    this.running = true;
    this.logger.info('Delivery worker started');
    while (this.running) {
      await this.processBatch();
      await this.sleep(1000);
    }
  }

  /** Stop the delivery worker */
  stop(): void {
    this.running = false;
    this.logger.info('Delivery worker stopped');
  }

  /** Process a single batch of notifications */
  async processBatch(): Promise<number> {
    const batch = this.queue.drain(this.batchSize);
    if (batch.length === 0) return 0;

    this.logger.debug('Processing batch', { size: batch.length });
    let delivered = 0;
    for (const notification of batch) {
      try {
        await this.notificationService.send({
          userId: notification.userId,
          channel: notification.channel,
          subject: notification.subject,
          body: notification.body,
        });
        delivered++;
      } catch (error) {
        this.logger.error('Delivery failed', { notificationId: notification.id });
      }
    }
    this.logger.info('Batch processed', { delivered, total: batch.length });
    return delivered;
  }

  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
