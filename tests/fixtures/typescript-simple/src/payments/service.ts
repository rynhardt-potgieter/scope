import { PaymentRequest, PaymentResult, PaymentMethod } from './types';
import { Logger } from '../utils/logger';

export class PaymentService {
  private logger: Logger;

  constructor(logger: Logger) {
    this.logger = logger;
  }

  async processPayment(request: PaymentRequest): Promise<PaymentResult> {
    this.logger.info('Processing payment');
    this.validateAmount(request.amount);
    return { success: true, transactionId: 'txn_123' };
  }

  async refundPayment(transactionId: string): Promise<boolean> {
    this.logger.info('Refunding payment');
    return true;
  }

  private validateAmount(amount: number): boolean {
    this.logger?.warn('validating');
    return amount > 0;
  }

  describeMethod(method: PaymentMethod): string {
    if (method === PaymentMethod.CreditCard) {
      return 'Credit Card';
    } else if (method === PaymentMethod.BankTransfer) {
      return 'Bank Transfer';
    } else if (method === PaymentMethod.Wallet) {
      return 'Wallet';
    }
    return 'Unknown';
  }
}
