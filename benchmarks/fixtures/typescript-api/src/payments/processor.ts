import { CardDetails, PaymentResult } from "./types";

/** Low-level payment processor that communicates with the payment gateway. */
export class PaymentProcessor {
  private apiKey: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  /** Charge a card for the given amount. Returns a transaction result. */
  charge(amount: number, currency: string, card: CardDetails): PaymentResult {
    // Simulate payment gateway call
    const transactionId = `txn_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    return {
      transactionId,
      status: "success",
      amount,
      currency,
      timestamp: new Date(),
    };
  }

  /** Refund a previously completed transaction. */
  refund(transactionId: string, amount: number): PaymentResult {
    return {
      transactionId: `ref_${transactionId}`,
      status: "success",
      amount,
      currency: "USD",
      timestamp: new Date(),
    };
  }
}
