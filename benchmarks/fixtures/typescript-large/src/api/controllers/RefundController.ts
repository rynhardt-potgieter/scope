import { Logger } from '../../shared/utils/Logger';
import { PaymentService } from '../../payments/services/PaymentService';
import { RefundService } from '../../payments/services/RefundService';
import { AuthGuard } from '../../auth/guards/AuthGuard';
import { ApiResponse } from '../../types/common';
import { PaymentResult } from '../../payments/types/PaymentTypes';
import { Refund, RefundReason } from '../../payments/models/Refund';
import { PaymentProcessor } from '../../types/enums';
import { createMoney, Currency } from '../../types/money';
import { CryptoService } from '../../shared/utils/Crypto';

/** Refund request payload */
export interface RefundRequest {
  paymentId: string;
  amount: number;
  currency: Currency;
  reason: RefundReason;
  notes?: string;
}

/** Partial refund with replacement payment request */
export interface PartialRefundRequest {
  originalPaymentId: string;
  refundAmount: number;
  replacementAmount: number;
  currency: Currency;
  reason: string;
}

/** Controller for refund endpoints */
export class RefundController {
  private paymentService: PaymentService;
  private refundService: RefundService;
  private authGuard: AuthGuard;
  private logger: Logger;

  constructor(paymentService: PaymentService, refundService: RefundService, authGuard: AuthGuard) {
    this.paymentService = paymentService;
    this.refundService = refundService;
    this.authGuard = authGuard;
    this.logger = new Logger('RefundController');
  }

  /** POST /refunds — process a full or partial refund */
  async processRefund(authHeader: string, request: RefundRequest): Promise<ApiResponse<Refund>> {
    const user = this.authGuard.guard(authHeader);
    const amount = createMoney(request.amount, request.currency);

    this.logger.info('Processing refund', { userId: user.sub, paymentId: request.paymentId });

    const refund = await this.refundService.processRefund(
      request.paymentId,
      amount,
      request.reason,
      user.sub,
      request.notes,
    );

    return {
      success: true,
      data: refund,
      message: 'Refund processed',
      timestamp: new Date(),
    };
  }

  /**
   * POST /refunds/partial — process a partial refund and create a replacement payment.
   * Refunds the original, then CALLS processPayment for the replacement charge.
   */
  async processPartialRefund(authHeader: string, request: PartialRefundRequest): Promise<ApiResponse<PaymentResult>> {
    const user = this.authGuard.guard(authHeader);
    this.logger.info('Processing partial refund with replacement', {
      userId: user.sub,
      originalPaymentId: request.originalPaymentId,
      refundAmount: request.refundAmount,
      replacementAmount: request.replacementAmount,
    });

    const refundAmount = createMoney(request.refundAmount, request.currency);
    await this.refundService.processRefund(
      request.originalPaymentId,
      refundAmount,
      RefundReason.CUSTOMER_REQUEST,
      user.sub,
      request.reason,
    );

    const replacementAmount = createMoney(request.replacementAmount, request.currency);
    const crypto = new CryptoService();

    const result = await this.paymentService.processPayment({
      userId: user.sub,
      amount: replacementAmount,
      processor: PaymentProcessor.STRIPE,
      description: `Replacement payment for refund on ${request.originalPaymentId}`,
      idempotencyKey: `partial_refund_${request.originalPaymentId}_${crypto.generateToken(8)}`,
      metadata: { originalPaymentId: request.originalPaymentId, refundAmount: request.refundAmount },
    });

    return {
      success: result.success,
      data: result,
      message: result.success ? 'Partial refund and replacement processed' : `Replacement payment failed: ${result.failureReason}`,
      timestamp: new Date(),
    };
  }
}
