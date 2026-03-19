import { Logger } from '../../shared/utils/Logger';
import { SubscriptionRepository } from '../repositories/SubscriptionRepository';
import { PaymentService } from './PaymentService';
import { NotificationService } from '../../notifications/services/NotificationService';
import { Subscription } from '../models/Subscription';
import { EntityId } from '../../types/common';
import { SubscriptionStatus, BillingInterval, PaymentProcessor } from '../../types/enums';
import { Money } from '../../types/money';
import { NotFoundError, ValidationError } from '../../types/errors';
import { DateUtils } from '../../shared/utils/DateUtils';

/** Service for subscription lifecycle management */
export class SubscriptionService {
  private subscriptionRepo: SubscriptionRepository;
  private paymentService: PaymentService;
  private notificationService: NotificationService;
  private logger: Logger;

  constructor(
    subscriptionRepo: SubscriptionRepository,
    paymentService: PaymentService,
    notificationService: NotificationService,
  ) {
    this.subscriptionRepo = subscriptionRepo;
    this.paymentService = paymentService;
    this.notificationService = notificationService;
    this.logger = new Logger('SubscriptionService');
  }

  /** Create a new subscription for a user */
  async createSubscription(
    userId: EntityId,
    planName: string,
    amount: Money,
    billingInterval: BillingInterval,
  ): Promise<Subscription> {
    const now = new Date();
    const periodEnd = this.calculatePeriodEnd(now, billingInterval);

    const subscription: Subscription = {
      id: `sub_${Date.now()}`,
      userId,
      planName,
      status: SubscriptionStatus.ACTIVE,
      billingInterval,
      amount,
      currentPeriodStart: now,
      currentPeriodEnd: periodEnd,
      cancelledAt: null,
      cancelReason: null,
      trialEndsAt: null,
      nextBillingDate: periodEnd,
      failedPaymentAttempts: 0,
      createdAt: now,
      updatedAt: now,
    };

    const saved = await this.subscriptionRepo.save(subscription);
    this.logger.info('Subscription created', { subscriptionId: saved.id, planName });

    await this.notificationService.send({
      userId,
      channel: 'email',
      subject: 'Subscription Activated',
      body: `Your ${planName} subscription is now active. Next billing date: ${DateUtils.formatDate(periodEnd)}`,
    });

    return saved;
  }

  /** Process a subscription renewal by charging the user */
  async processRenewal(subscriptionId: EntityId): Promise<Subscription> {
    const subscription = await this.subscriptionRepo.findById(subscriptionId);
    if (!subscription) {
      throw new NotFoundError('Subscription', subscriptionId);
    }
    if (subscription.status !== SubscriptionStatus.ACTIVE && subscription.status !== SubscriptionStatus.PAST_DUE) {
      throw new ValidationError(`Cannot renew subscription with status ${subscription.status}`);
    }

    this.logger.info('Processing subscription renewal', { subscriptionId, amount: subscription.amount.amount });

    let paymentResult;
    try {
      paymentResult = await this.paymentService.processPayment(
        subscription.userId,
        subscription.amount,
        PaymentProcessor.STRIPE,
        `Subscription renewal: ${subscription.planName}`,
        `sub_renewal_${subscriptionId}_${Date.now()}`,
      );
    } catch (error) {
      await this.subscriptionRepo.incrementFailedAttempts(subscriptionId);
      const message = error instanceof Error ? error.message : 'Unknown payment error';
      this.logger.error('Payment processing error during renewal', { subscriptionId, error: message });
      throw new ValidationError(`Renewal payment failed: ${message}`);
    }

    if (paymentResult.success) {
      const newPeriodStart = subscription.currentPeriodEnd;
      const newPeriodEnd = this.calculatePeriodEnd(newPeriodStart, subscription.billingInterval);

      const updated = await this.subscriptionRepo.updatePeriod(
        subscriptionId,
        newPeriodStart,
        newPeriodEnd,
      );
      this.logger.info('Subscription renewed', { subscriptionId });
      return updated!;
    } else {
      await this.subscriptionRepo.incrementFailedAttempts(subscriptionId);
      this.logger.warn('Subscription renewal payment failed', {
        subscriptionId,
        reason: paymentResult.failureReason,
      });
      throw new ValidationError(`Renewal payment failed: ${paymentResult.failureReason}`);
    }
  }

  /** Cancel a subscription */
  async cancelSubscription(subscriptionId: EntityId, reason: string): Promise<Subscription> {
    const subscription = await this.subscriptionRepo.findById(subscriptionId);
    if (!subscription) {
      throw new NotFoundError('Subscription', subscriptionId);
    }

    const updated = await this.subscriptionRepo.cancel(subscriptionId, reason);
    this.logger.info('Subscription cancelled', { subscriptionId, reason });

    await this.notificationService.send({
      userId: subscription.userId,
      channel: 'email',
      subject: 'Subscription Cancelled',
      body: `Your ${subscription.planName} subscription has been cancelled. You will retain access until ${DateUtils.formatDate(subscription.currentPeriodEnd)}.`,
    });

    return updated!;
  }

  /** Get a subscription by ID */
  async getSubscription(subscriptionId: EntityId): Promise<Subscription> {
    const subscription = await this.subscriptionRepo.findById(subscriptionId);
    if (!subscription) {
      throw new NotFoundError('Subscription', subscriptionId);
    }
    return subscription;
  }

  private calculatePeriodEnd(start: Date, interval: BillingInterval): Date {
    switch (interval) {
      case BillingInterval.MONTHLY:
        return DateUtils.addDays(start, 30);
      case BillingInterval.QUARTERLY:
        return DateUtils.addDays(start, 90);
      case BillingInterval.YEARLY:
        return DateUtils.addDays(start, 365);
    }
  }
}
