import { Logger } from '../../shared/utils/Logger';
import { PaymentService } from '../../payments/services/PaymentService';
import { PaymentMapper } from '../../payments/mappers/PaymentMapper';
import { AuthGuard } from '../../auth/guards/AuthGuard';
import { ApiResponse } from '../../types/common';
import { PaymentResponse, CreatePaymentRequest } from '../../payments/dtos/PaymentDtos';
import { PaymentProcessor } from '../../types/enums';
import { createMoney, Currency } from '../../types/money';

/** Controller for direct payment endpoints */
export class PaymentController {
  private paymentService: PaymentService;
  private authGuard: AuthGuard;
  private logger: Logger;

  constructor(paymentService: PaymentService, authGuard: AuthGuard) {
    this.paymentService = paymentService;
    this.authGuard = authGuard;
    this.logger = new Logger('PaymentController');
  }

  /** POST /payments — Note: does NOT call processPayment directly; delegates to PaymentService methods */
  async createPayment(
    authHeader: string,
    request: CreatePaymentRequest,
  ): Promise<ApiResponse<PaymentResponse>> {
    const user = this.authGuard.guard(authHeader);
    this.logger.info('Create payment request', { userId: user.sub });

    const isValid = await this.paymentService.validateCard(request.cardToken);
    if (!isValid) {
      return {
        success: false,
        data: { paymentId: '', status: 'failed', processorTransactionId: null, amount: request.amount, currency: request.currency, createdAt: new Date().toISOString() },
        message: 'Invalid card',
        timestamp: new Date(),
      };
    }

    return {
      success: true,
      data: {
        paymentId: '',
        status: 'pending',
        processorTransactionId: null,
        amount: request.amount,
        currency: request.currency,
        createdAt: new Date().toISOString(),
      },
      message: 'Payment initiated',
      timestamp: new Date(),
    };
  }

  /** GET /payments/:id */
  async getPayment(authHeader: string, paymentId: string): Promise<ApiResponse<PaymentResponse>> {
    this.authGuard.guard(authHeader);
    const payment = await this.paymentService.getTransaction(paymentId);
    return {
      success: true,
      data: {
        paymentId: payment.id,
        status: payment.status,
        processorTransactionId: payment.processorTransactionId,
        amount: payment.amount.amount,
        currency: payment.amount.currency,
        createdAt: payment.createdAt.toISOString(),
      },
      message: 'Payment retrieved',
      timestamp: new Date(),
    };
  }

  /** GET /payments — list payments for the authenticated user */
  async listPayments(authHeader: string): Promise<ApiResponse<PaymentResponse[]>> {
    const user = this.authGuard.guard(authHeader);
    this.logger.info('List payments', { userId: user.sub });
    return {
      success: true,
      data: [],
      message: 'Payments listed',
      timestamp: new Date(),
    };
  }
}
