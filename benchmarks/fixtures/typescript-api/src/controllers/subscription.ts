import { PaymentService } from "../payments/service";
import { CardDetails, PaymentResult } from "../payments/types";

/** Service managing subscription lifecycle and recurring payments. */
export class SubscriptionService {
  private paymentService: PaymentService;

  constructor(paymentService: PaymentService) {
    this.paymentService = paymentService;
  }

  /** Renew an existing subscription by charging the saved card. */
  renew(userId: string, amount: number, card: CardDetails): PaymentResult {
    // Call processPayment — caller #3
    const result = this.paymentService.processPayment(amount, userId, card);
    return result;
  }

  /** Upgrade a subscription to a higher tier, charging the price difference. */
  upgrade(userId: string, priceDifference: number, card: CardDetails): PaymentResult {
    // Call processPayment — caller #4
    const result = this.paymentService.processPayment(priceDifference, userId, card);
    return result;
  }
}
