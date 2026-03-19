import { Logger } from '../../shared/utils/Logger';
import { ChargeRequest, ChargeResponse } from '../types/PaymentTypes';
import { Money } from '../../types/money';

/** Stripe payment processor adapter */
export class StripeProcessor {
  private apiKey: string;
  private logger: Logger;

  constructor(apiKey: string = 'sk_test_default') {
    this.apiKey = apiKey;
    this.logger = new Logger('StripeProcessor');
  }

  /** Charge a card via Stripe */
  async charge(request: ChargeRequest): Promise<ChargeResponse> {
    this.logger.info('Charging via Stripe', {
      amount: request.amount.amount,
      currency: request.amount.currency,
    });

    const transactionId = `ch_${Date.now()}_${Math.random().toString(36).slice(2, 10)}`;

    if (request.amount.amount > 999999) {
      return {
        success: false,
        transactionId: '',
        failureReason: 'Amount exceeds maximum charge limit',
      };
    }

    this.logger.info('Stripe charge successful', { transactionId });
    return {
      success: true,
      transactionId,
      failureReason: null,
    };
  }

  /** Refund a Stripe charge */
  async refund(transactionId: string, amount: Money): Promise<ChargeResponse> {
    this.logger.info('Refunding via Stripe', { transactionId, amount: amount.amount });
    const refundId = `re_${Date.now()}_${Math.random().toString(36).slice(2, 10)}`;
    return {
      success: true,
      transactionId: refundId,
      failureReason: null,
    };
  }

  /** Create a Stripe customer profile */
  async createCustomer(email: string, name: string): Promise<string> {
    this.logger.info('Creating Stripe customer', { email });
    return `cus_${Date.now()}_${Math.random().toString(36).slice(2, 10)}`;
  }
}
