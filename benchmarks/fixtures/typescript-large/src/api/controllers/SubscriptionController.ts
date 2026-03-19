import { Logger } from '../../shared/utils/Logger';
import { PaymentService } from '../../payments/services/PaymentService';
import { SubscriptionService } from '../../payments/services/SubscriptionService';
import { AuthGuard } from '../../auth/guards/AuthGuard';
import { ApiResponse } from '../../types/common';
import { PaymentResult } from '../../payments/types/PaymentTypes';
import { Subscription } from '../../payments/models/Subscription';
import { SubscriptionResponse, CreateSubscriptionRequest, CancelSubscriptionRequest } from '../../payments/dtos/SubscriptionDtos';
import { PaymentProcessor, BillingInterval } from '../../types/enums';
import { createMoney, Currency } from '../../types/money';
import { CryptoService } from '../../shared/utils/Crypto';

/** Controller for subscription endpoints */
export class SubscriptionController {
  private paymentService: PaymentService;
  private subscriptionService: SubscriptionService;
  private authGuard: AuthGuard;
  private logger: Logger;

  constructor(
    paymentService: PaymentService,
    subscriptionService: SubscriptionService,
    authGuard: AuthGuard,
  ) {
    this.paymentService = paymentService;
    this.subscriptionService = subscriptionService;
    this.authGuard = authGuard;
    this.logger = new Logger('SubscriptionController');
  }

  /** POST /subscriptions — create a new subscription */
  async create(authHeader: string, request: CreateSubscriptionRequest): Promise<ApiResponse<SubscriptionResponse>> {
    const user = this.authGuard.guard(authHeader);
    const amount = createMoney(request.amount, request.currency as Currency);

    const subscription = await this.subscriptionService.createSubscription(
      user.sub,
      request.planName,
      amount,
      request.billingInterval as BillingInterval,
    );

    return {
      success: true,
      data: {
        subscriptionId: subscription.id,
        planName: subscription.planName,
        status: subscription.status,
        billingInterval: subscription.billingInterval,
        amount: subscription.amount.amount,
        currency: subscription.amount.currency,
        currentPeriodStart: subscription.currentPeriodStart.toISOString(),
        currentPeriodEnd: subscription.currentPeriodEnd.toISOString(),
        nextBillingDate: subscription.nextBillingDate.toISOString(),
        createdAt: subscription.createdAt.toISOString(),
      },
      message: 'Subscription created',
      timestamp: new Date(),
    };
  }

  /**
   * POST /subscriptions/:id/renew
   * Manually triggers a subscription renewal.
   * CALLS processPayment (via SubscriptionService OR directly).
   */
  async renewSubscription(authHeader: string, subscriptionId: string): Promise<ApiResponse<PaymentResult>> {
    const user = this.authGuard.guard(authHeader);
    this.logger.info('Renewing subscription', { userId: user.sub, subscriptionId });

    const subscription = await this.subscriptionService.getSubscription(subscriptionId);
    const crypto = new CryptoService();

    const result = await this.paymentService.processPayment({
      userId: user.sub,
      amount: subscription.amount,
      processor: PaymentProcessor.STRIPE,
      description: `Manual renewal: ${subscription.planName}`,
      idempotencyKey: `manual_renew_${subscriptionId}_${crypto.generateToken(8)}`,
      metadata: { subscriptionId, planName: subscription.planName },
    });

    this.logger.info('Subscription renewal result', { subscriptionId, success: result.success });

    return {
      success: result.success,
      data: result,
      message: result.success ? 'Subscription renewed' : `Renewal failed: ${result.failureReason}`,
      timestamp: new Date(),
    };
  }

  /** POST /subscriptions/:id/cancel */
  async cancel(authHeader: string, subscriptionId: string, request: CancelSubscriptionRequest): Promise<ApiResponse<{ cancelled: boolean }>> {
    const user = this.authGuard.guard(authHeader);
    await this.subscriptionService.cancelSubscription(subscriptionId, request.reason);
    return {
      success: true,
      data: { cancelled: true },
      message: 'Subscription cancelled',
      timestamp: new Date(),
    };
  }
}
