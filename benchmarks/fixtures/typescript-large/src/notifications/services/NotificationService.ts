import { Logger } from '../../shared/utils/Logger';
import { NotificationRepository } from '../repositories/NotificationRepository';
import { EmailService } from './EmailService';
import { PushService } from './PushService';
import { SmsService } from './SmsService';
import { Notification, DeliveryStatus } from '../models/Notification';
import { EntityId } from '../../types/common';
import { NotificationChannel } from '../../types/enums';
import { NOTIFICATION_DELIVERY_TIMEOUT_MS } from '../../types/constants';

/** Notification send request */
export interface SendNotificationRequest {
  userId: EntityId;
  channel: string;
  subject: string;
  body: string;
  metadata?: Record<string, unknown>;
}

/** Multi-channel notification orchestrator */
export class NotificationService {
  private notificationRepo: NotificationRepository;
  private emailService: EmailService;
  private pushService: PushService;
  private smsService: SmsService;
  private logger: Logger;

  constructor(
    notificationRepo: NotificationRepository,
    emailService: EmailService,
    pushService: PushService,
    smsService: SmsService,
  ) {
    this.notificationRepo = notificationRepo;
    this.emailService = emailService;
    this.pushService = pushService;
    this.smsService = smsService;
    this.logger = new Logger('NotificationService');
  }

  /** Send a notification through the specified channel */
  async send(request: SendNotificationRequest): Promise<Notification> {
    const channel = request.channel as NotificationChannel;
    const notification: Notification = {
      id: `ntf_${Date.now()}`,
      userId: request.userId,
      channel,
      subject: request.subject,
      body: request.body,
      status: DeliveryStatus.PENDING,
      sentAt: null,
      deliveredAt: null,
      failureReason: null,
      retryCount: 0,
      metadata: request.metadata ?? {},
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    const saved = await this.notificationRepo.save(notification);

    try {
      await this.deliver(saved);
      await this.notificationRepo.updateStatus(saved.id, DeliveryStatus.SENT);
      this.logger.info('Notification sent', { notificationId: saved.id, channel });
    } catch (error) {
      const reason = error instanceof Error ? error.message : 'Unknown error';
      await this.notificationRepo.updateStatus(saved.id, DeliveryStatus.FAILED, reason);
      this.logger.error('Notification delivery failed', { notificationId: saved.id, reason });
    }

    return saved;
  }

  /** Send notifications to multiple users */
  async sendBulk(userIds: EntityId[], channel: string, subject: string, body: string): Promise<number> {
    let sent = 0;
    for (const userId of userIds) {
      try {
        await this.send({ userId, channel, subject, body });
        sent++;
      } catch (error) {
        this.logger.error('Bulk notification failed for user', { userId });
      }
    }
    this.logger.info('Bulk send complete', { total: userIds.length, sent });
    return sent;
  }

  /** Get notification history for a user */
  async getHistory(userId: EntityId, limit: number = 20): Promise<Notification[]> {
    return this.notificationRepo.findByUser(userId, limit);
  }

  private async deliver(notification: Notification): Promise<void> {
    switch (notification.channel) {
      case NotificationChannel.EMAIL:
        await this.emailService.sendEmail(notification.userId, notification.subject, notification.body);
        break;
      case NotificationChannel.PUSH:
        await this.pushService.sendPush(notification.userId, notification.subject, notification.body);
        break;
      case NotificationChannel.SMS:
        await this.smsService.sendSms(notification.userId, notification.body);
        break;
      default:
        this.logger.warn('Unsupported notification channel', { channel: notification.channel });
    }
  }
}
