import { Logger } from '../../shared/utils/Logger';
import { PaymentService } from '../../payments/services/PaymentService';
import { PaymentRetryQueue } from '../../payments/workers/PaymentRetryQueue';
import { PaymentRepository } from '../../payments/repositories/PaymentRepository';
import { PaymentProcessor } from '../../types/enums';
import { CryptoService } from '../../shared/utils/Crypto';

/**
 * Background worker that retries failed payments from the retry queue.
 * CALLS processPayment for each retry attempt.
 */
export class PaymentRetryWorker {
  private paymentService: PaymentService;
  private retryQueue: PaymentRetryQueue;
  private paymentRepo: PaymentRepository;
  private logger: Logger;

  constructor(
    paymentService: PaymentService,
    retryQueue: PaymentRetryQueue,
    paymentRepo: PaymentRepository,
  ) {
    this.paymentService = paymentService;
    this.retryQueue = retryQueue;
    this.paymentRepo = paymentRepo;
    this.logger = new Logger('PaymentRetryWorker');
  }

  /** Process the next retry job from the queue */
  async retryFailedPayment(): Promise<void> {
    const job = this.retryQueue.dequeue();
    if (!job) {
      this.logger.debug('No retry jobs in queue');
      return;
    }

    this.logger.info('Retrying failed payment', {
      paymentId: job.paymentId,
      attempt: job.attempt,
      reason: job.reason,
    });

    const payment = await this.paymentRepo.findById(job.paymentId);
    if (!payment) {
      this.logger.warn('Payment not found for retry', { paymentId: job.paymentId });
      return;
    }

    const crypto = new CryptoService();
    const result = await this.paymentService.processPayment(
      payment.userId,
      payment.amount,
      payment.processor as PaymentProcessor,
      `Retry ${job.attempt}: ${payment.description}`,
      `retry_${job.paymentId}_${job.attempt}_${crypto.generateToken(8)}`,
      { originalPaymentId: job.paymentId, retryAttempt: job.attempt },
    );

    if (result.success) {
      this.logger.info('Payment retry successful', { paymentId: job.paymentId, attempt: job.attempt });
    } else {
      this.logger.warn('Payment retry failed', {
        paymentId: job.paymentId,
        attempt: job.attempt,
        reason: result.failureReason,
      });
      this.retryQueue.enqueue(job.paymentId, result.failureReason ?? 'Unknown', job.attempt + 1);
    }
  }

  /** Process all ready jobs in the queue */
  async processAll(): Promise<number> {
    let processed = 0;
    while (this.retryQueue.size() > 0) {
      await this.retryFailedPayment();
      processed++;
    }
    this.logger.info('Retry batch complete', { processed });
    return processed;
  }
}
