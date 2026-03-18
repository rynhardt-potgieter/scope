import { PaymentRequest, PaymentResult } from './types';
import { Logger } from '../utils/logger';

export class PaymentService {
  private logger: Logger;

  constructor(logger: Logger) {
    this.logger = logger;
  }

  async processPayment(request: PaymentRequest): Promise<PaymentResult> {
    this.logger.info('Processing payment');
    return { success: true, transactionId: 'txn_123' };
  }

  async refundPayment(transactionId: string): Promise<boolean> {
    this.logger.info('Refunding payment');
    return true;
  }

  private validateAmount(amount: number): boolean {
    return amount > 0;
  }
}
