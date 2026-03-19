import { Logger } from '../../shared/utils/Logger';
import { SubscriptionRepository } from '../../payments/repositories/SubscriptionRepository';
import { SubscriptionService } from '../../payments/services/SubscriptionService';
import { NotificationService } from '../../notifications/services/NotificationService';
import { DateUtils } from '../../shared/utils/DateUtils';

/** Background worker that checks for expiring subscriptions */
export class SubscriptionRenewalWorker {
  private subscriptionRepo: SubscriptionRepository;
  private subscriptionService: SubscriptionService;
  private notificationService: NotificationService;
  private logger: Logger;

  constructor(
    subscriptionRepo: SubscriptionRepository,
    subscriptionService: SubscriptionService,
    notificationService: NotificationService,
  ) {
    this.subscriptionRepo = subscriptionRepo;
    this.subscriptionService = subscriptionService;
    this.notificationService = notificationService;
    this.logger = new Logger('SubscriptionRenewalWorker');
  }

  /** Check for subscriptions expiring in the next 3 days and send reminders */
  async checkExpiringSubscriptions(): Promise<number> {
    this.logger.info('Checking for expiring subscriptions');
    const active = await this.subscriptionRepo.findActive();
    const threeDaysFromNow = DateUtils.addDays(new Date(), 3);
    let remindersSent = 0;

    for (const subscription of active) {
      if (subscription.currentPeriodEnd <= threeDaysFromNow) {
        try {
          await this.notificationService.send({
            userId: subscription.userId,
            channel: 'email',
            subject: 'Subscription Renewal Reminder',
            body: `Your ${subscription.planName} subscription renews on ${DateUtils.formatDate(subscription.nextBillingDate)}.`,
          });
          remindersSent++;
        } catch (error) {
          this.logger.error('Failed to send renewal reminder', { subscriptionId: subscription.id });
        }
      }
    }

    this.logger.info('Expiring subscription check complete', { checked: active.length, remindersSent });
    return remindersSent;
  }

  /** Auto-renew subscriptions that are past their billing date */
  async autoRenewDue(): Promise<number> {
    const active = await this.subscriptionRepo.findActive();
    const now = new Date();
    let renewed = 0;

    for (const subscription of active) {
      if (subscription.nextBillingDate <= now) {
        try {
          await this.subscriptionService.processRenewal(subscription.id);
          renewed++;
        } catch (error) {
          this.logger.error('Auto-renewal failed', { subscriptionId: subscription.id });
        }
      }
    }

    this.logger.info('Auto-renewal complete', { renewed });
    return renewed;
  }
}
