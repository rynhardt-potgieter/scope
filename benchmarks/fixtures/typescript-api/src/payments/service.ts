import { CardDetails, PaymentRequest, PaymentResult } from "./types";
import { PaymentProcessor } from "./processor";

/** High-level payment service used by controllers and workers. */
export class PaymentService {
  private processor: PaymentProcessor;
  private transactions: Map<string, PaymentResult>;

  constructor(processor: PaymentProcessor) {
    this.processor = processor;
    this.transactions = new Map();
  }

  /**
   * Process a payment for the given amount and user.
   * This is the primary entry point for all payment operations.
   */
  processPayment(amount: number, userId: string, card: CardDetails): PaymentResult {
    this.validateCard(card);

    const result = this.processor.charge(amount, "USD", card);
    this.transactions.set(result.transactionId, result);
    return result;
  }

  /** Refund a previously completed payment by transaction ID. */
  refundPayment(transactionId: string): PaymentResult {
    const original = this.getTransaction(transactionId);
    if (!original) {
      return {
        transactionId: "",
        status: "failed",
        amount: 0,
        currency: "USD",
        timestamp: new Date(),
        errorMessage: `Transaction ${transactionId} not found`,
      };
    }

    const result = this.processor.refund(transactionId, original.amount);
    this.transactions.set(result.transactionId, result);
    return result;
  }

  /** Validate card details before processing. */
  validateCard(card: CardDetails): void {
    if (!card.cardNumber || card.cardNumber.length < 13) {
      throw new Error("Invalid card number");
    }
    if (!card.cvv || card.cvv.length < 3) {
      throw new Error("Invalid CVV");
    }
    if (card.expiryYear < new Date().getFullYear()) {
      throw new Error("Card expired");
    }
  }

  /** Retrieve a transaction by its ID. */
  getTransaction(transactionId: string): PaymentResult | undefined {
    return this.transactions.get(transactionId);
  }
}
