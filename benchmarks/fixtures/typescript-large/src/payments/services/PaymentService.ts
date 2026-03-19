import { Logger } from '../../shared/utils/Logger';
import { CacheService } from '../../shared/utils/Cache';
import { PaymentRepository } from '../repositories/PaymentRepository';
import { ProcessorFactory } from '../processors/ProcessorFactory';
import { PaymentValidator } from '../validators/PaymentValidator';
import { Payment } from '../models/Payment';
import { PaymentResult } from '../types/PaymentTypes';
import { PaymentStatus, PaymentProcessor } from '../../types/enums';
import { EntityId } from '../../types/common';
import { NotFoundError, ValidationError } from '../../types/errors';
import { Money } from '../../types/money';
import { NotificationService } from '../../notifications/services/NotificationService';

/** Core payment processing service — the central hub for all payment operations */
export class PaymentService {
  private paymentRepo: PaymentRepository;
  private processorFactory: ProcessorFactory;
  private validator: PaymentValidator;
  private notificationService: NotificationService;
  private cache: CacheService;
  private logger: Logger;

  constructor(
    paymentRepo: PaymentRepository,
    processorFactory: ProcessorFactory,
    validator: PaymentValidator,
    notificationService: NotificationService,
    cache: CacheService,
  ) {
    this.paymentRepo = paymentRepo;
    this.processorFactory = processorFactory;
    this.validator = validator;
    this.notificationService = notificationService;
    this.cache = cache;
    this.logger = new Logger('PaymentService');
  }

  /**
   * Process a payment through the configured payment processor.
   *
   * This is the primary entry point for all payment operations in the system.
   * Called by OrderController.checkout, OrderController.retryPayment,
   * SubscriptionController.renewSubscription, SubscriptionService.processRenewal,
   * PaymentRetryWorker.retryFailedPayment, InvoiceService.settleInvoice,
   * and RefundController.processPartialRefund.
   */
  async processPayment(
    userId: EntityId,
    amount: Money,
    processor: PaymentProcessor,
    description: string,
    idempotencyKey: string,
    metadata?: Record<string, unknown>,
  ): Promise<PaymentResult> {
    this.logger.info('Processing payment', {
      userId,
      amount: amount.amount,
      currency: amount.currency,
      processor,
    });

    this.validator.validateAmount(amount);
    this.validator.validateCurrency(amount.currency);

    const proc = this.processorFactory.getProcessor(processor);

    const payment = await this.paymentRepo.save({
      id: `pay_${Date.now()}`,
      userId,
      amount,
      status: PaymentStatus.PROCESSING,
      processor,
      processorTransactionId: null,
      description,
      metadata: metadata ?? {},
      failureReason: null,
      refundedAmount: null,
      completedAt: null,
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    try {
      const chargeResult = await proc.charge({
        amount,
        cardToken: 'tok_default',
        description,
        idempotencyKey,
        metadata,
      });

      if (chargeResult.success) {
        await this.paymentRepo.updateStatus(payment.id, PaymentStatus.COMPLETED, chargeResult.transactionId);
        this.cache.invalidatePrefix(`payment:${userId}`);

        await this.notificationService.send({
          userId,
          channel: 'email',
          subject: 'Payment Confirmation',
          body: `Your payment of ${amount.amount} ${amount.currency} has been processed.`,
        });

        this.logger.info('Payment successful', { paymentId: payment.id });
        return {
          success: true,
          paymentId: payment.id,
          processorTransactionId: chargeResult.transactionId,
          status: PaymentStatus.COMPLETED,
          failureReason: null,
        };
      } else {
        await this.paymentRepo.updateStatus(payment.id, PaymentStatus.FAILED, null, chargeResult.failureReason);
        this.logger.warn('Payment failed', { paymentId: payment.id, reason: chargeResult.failureReason });
        return {
          success: false,
          paymentId: payment.id,
          processorTransactionId: null,
          status: PaymentStatus.FAILED,
          failureReason: chargeResult.failureReason,
        };
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      await this.paymentRepo.updateStatus(payment.id, PaymentStatus.FAILED, null, message);
      this.logger.error('Payment processing error', { paymentId: payment.id, error: message });
      return {
        success: false,
        paymentId: payment.id,
        processorTransactionId: null,
        status: PaymentStatus.FAILED,
        failureReason: message,
      };
    }
  }

  /** Refund a previously completed payment */
  async refundPayment(paymentId: EntityId, amount: Money, reason: string): Promise<PaymentResult> {
    const payment = await this.paymentRepo.findById(paymentId);
    if (!payment) {
      throw new NotFoundError('Payment', paymentId);
    }
    if (payment.status !== PaymentStatus.COMPLETED) {
      throw new ValidationError('Can only refund completed payments');
    }

    this.logger.info('Processing refund', { paymentId, amount: amount.amount });
    const processor = this.processorFactory.getProcessor(payment.processor as any);
    await processor.refund(payment.processorTransactionId!, amount);
    await this.paymentRepo.updateStatus(paymentId, PaymentStatus.REFUNDED);
    this.cache.invalidatePrefix(`payment:${payment.userId}`);

    return {
      success: true,
      paymentId,
      processorTransactionId: payment.processorTransactionId,
      status: PaymentStatus.REFUNDED,
      failureReason: null,
    };
  }

  /** Validate a card token before processing */
  async validateCard(cardToken: string): Promise<boolean> {
    this.logger.debug('Validating card', { cardToken: cardToken.slice(0, 8) + '...' });
    return cardToken.startsWith('tok_') && cardToken.length > 8;
  }

  /** Retrieve a payment transaction by ID */
  async getTransaction(paymentId: EntityId): Promise<Payment> {
    const cached = this.cache.get<Payment>(`payment:${paymentId}`);
    if (cached) return cached;

    const payment = await this.paymentRepo.findById(paymentId);
    if (!payment) {
      throw new NotFoundError('Payment', paymentId);
    }
    this.cache.set(`payment:${paymentId}`, payment);
    return payment;
  }
}
