import { StripeProcessor } from './StripeProcessor';
import { PayPalProcessor } from './PayPalProcessor';
import { PaymentProcessor } from '../../types/enums';
import { ValidationError } from '../../types/errors';

/** Processor interface shared by all payment processors */
export interface PaymentProcessorAdapter {
  charge(request: import('../types/PaymentTypes').ChargeRequest): Promise<import('../types/PaymentTypes').ChargeResponse>;
  refund(transactionId: string, amount: import('../../types/money').Money): Promise<import('../types/PaymentTypes').ChargeResponse>;
}

/** Factory for creating payment processor instances */
export class ProcessorFactory {
  private stripe: StripeProcessor;
  private paypal: PayPalProcessor;

  constructor() {
    this.stripe = new StripeProcessor();
    this.paypal = new PayPalProcessor();
  }

  /** Get the appropriate processor adapter for the given type */
  getProcessor(type: PaymentProcessor): PaymentProcessorAdapter {
    try {
      switch (type) {
        case PaymentProcessor.STRIPE:
          return this.stripe;
        case PaymentProcessor.PAYPAL:
          return this.paypal;
        default:
          throw new ValidationError(`Unknown payment processor: ${type}`);
      }
    } catch (error) {
      if (error instanceof ValidationError) throw error;
      throw new ValidationError(`Failed to initialize processor: ${type}`);
    }
  }
}
