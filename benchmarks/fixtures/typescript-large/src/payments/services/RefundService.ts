import { Logger } from '../../shared/utils/Logger';
import { PaymentRepository } from '../repositories/PaymentRepository';
import { ProcessorFactory } from '../processors/ProcessorFactory';
import { Refund, RefundReason, RefundStatus } from '../models/Refund';
import { EntityId } from '../../types/common';
import { Money, subtractMoney, createMoney } from '../../types/money';
import { PaymentStatus } from '../../types/enums';
import { NotFoundError, ValidationError } from '../../types/errors';

/** Service for processing refunds */
export class RefundService {
  private paymentRepo: PaymentRepository;
  private processorFactory: ProcessorFactory;
  private logger: Logger;

  constructor(paymentRepo: PaymentRepository, processorFactory: ProcessorFactory) {
    this.paymentRepo = paymentRepo;
    this.processorFactory = processorFactory;
    this.logger = new Logger('RefundService');
  }

  /** Process a refund for a completed payment */
  async processRefund(
    paymentId: EntityId,
    amount: Money,
    reason: RefundReason,
    processedBy: EntityId,
    notes: string = '',
  ): Promise<Refund> {
    const payment = await this.paymentRepo.findById(paymentId);
    if (!payment) {
      throw new NotFoundError('Payment', paymentId);
    }
    if (payment.status !== PaymentStatus.COMPLETED && payment.status !== PaymentStatus.PARTIALLY_REFUNDED) {
      throw new ValidationError('Can only refund completed or partially refunded payments');
    }
    if (amount.amount > payment.amount.amount) {
      throw new ValidationError('Refund amount exceeds payment amount');
    }

    this.logger.info('Processing refund', { paymentId, amount: amount.amount, reason });

    const processor = this.processorFactory.getProcessor(payment.processor as any);
    const refundResult = await processor.refund(payment.processorTransactionId!, amount);

    const refund: Refund = {
      id: `ref_${Date.now()}`,
      paymentId,
      userId: payment.userId,
      amount,
      reason,
      status: RefundStatus.COMPLETED,
      processorRefundId: refundResult.transactionId,
      notes,
      processedAt: new Date(),
      processedBy,
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    const isFullRefund = amount.amount >= payment.amount.amount;
    const newStatus = isFullRefund ? PaymentStatus.REFUNDED : PaymentStatus.PARTIALLY_REFUNDED;
    await this.paymentRepo.updateStatus(paymentId, newStatus);

    this.logger.info('Refund processed', { refundId: refund.id, paymentId, status: newStatus });
    return refund;
  }

  /** Calculate the maximum refundable amount for a payment */
  calculateRefundAmount(payment: { amount: Money; refundedAmount: Money | null }): Money {
    if (!payment.refundedAmount) {
      return payment.amount;
    }
    return subtractMoney(payment.amount, payment.refundedAmount);
  }
}
