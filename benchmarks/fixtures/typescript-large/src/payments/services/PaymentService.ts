import { Logger } from '../../shared/utils/Logger';
import { CacheService } from '../../shared/utils/Cache';
import { PaymentRepository } from '../repositories/PaymentRepository';
import { ProcessorFactory } from '../processors/ProcessorFactory';
import { PaymentValidator } from '../validators/PaymentValidator';
import { Payment } from '../models/Payment';
import { PaymentRequest, PaymentResult } from '../types/PaymentTypes';
import { PaymentStatus } from '../../types/enums';
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
  async processPayment(request: PaymentRequest): Promise<PaymentResult> {
    this.logger.info('Processing payment', {
      userId: request.userId,
      amount: request.amount.amount,
      currency: request.amount.currency,
      processor: request.processor,
    });

    this.validator.validateAmount(request.amount);
    this.validator.validateCurrency(request.amount.currency);

    const processor = this.processorFactory.getProcessor(request.processor);

    const payment = await this.paymentRepo.save({
      id: `pay_${Date.now()}`,
      userId: request.userId,
      amount: request.amount,
      status: PaymentStatus.PROCESSING,
      processor: request.processor,
      processorTransactionId: null,
      description: request.description,
      metadata: request.metadata ?? {},
      failureReason: null,
      refundedAmount: null,
      completedAt: null,
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    try {
      const chargeResult = await processor.charge({
        amount: request.amount,
        cardToken: 'tok_default',
        description: request.description,
        idempotencyKey: request.idempotencyKey,
        metadata: request.metadata,
      });

      if (chargeResult.success) {
        await this.paymentRepo.updateStatus(payment.id, PaymentStatus.COMPLETED, chargeResult.transactionId);
        this.cache.invalidatePrefix(`payment:${request.userId}`);

        await this.notificationService.send({
          userId: request.userId,
          channel: 'email',
          subject: 'Payment Confirmation',
          body: `Your payment of ${request.amount.amount} ${request.amount.currency} has been processed.`,
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
