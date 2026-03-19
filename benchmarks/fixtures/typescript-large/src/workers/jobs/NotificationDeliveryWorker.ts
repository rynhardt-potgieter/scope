import { Logger } from '../../shared/utils/Logger';
import { NotificationQueue } from '../../notifications/queues/NotificationQueue';
import { NotificationService } from '../../notifications/services/NotificationService';
import { NotificationRepository } from '../../notifications/repositories/NotificationRepository';
import { DeliveryStatus } from '../../notifications/models/Notification';

/** Background worker that processes the notification delivery queue */
export class NotificationDeliveryWorker {
  private queue: NotificationQueue;
  private notificationService: NotificationService;
  private notificationRepo: NotificationRepository;
  private logger: Logger;
  private batchSize: number;

  constructor(
    queue: NotificationQueue,
    notificationService: NotificationService,
    notificationRepo: NotificationRepository,
    batchSize: number = 20,
  ) {
    this.queue = queue;
    this.notificationService = notificationService;
    this.notificationRepo = notificationRepo;
    this.logger = new Logger('NotificationDeliveryWorker');
    this.batchSize = batchSize;
  }

  /** Process the next batch of queued notifications */
  async processNotificationQueue(): Promise<number> {
    const batch = this.queue.drain(this.batchSize);
    if (batch.length === 0) {
      this.logger.debug('Notification queue empty');
      return 0;
    }

    this.logger.info('Processing notification batch', { size: batch.length });
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
        const reason = error instanceof Error ? error.message : 'Unknown error';
        await this.notificationRepo.updateStatus(notification.id, DeliveryStatus.FAILED, reason);
        this.logger.error('Notification delivery failed', { notificationId: notification.id, reason });
      }
    }

    this.logger.info('Notification batch processed', { delivered, total: batch.length });
    return delivered;
  }
}
