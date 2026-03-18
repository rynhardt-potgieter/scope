import { PaymentService } from "../payments/service";
import { CardDetails, PaymentResult } from "../payments/types";

/** Background worker that retries failed payments on a schedule. */
export class PaymentRetryWorker {
  private paymentService: PaymentService;
  private maxRetries: number;

  constructor(paymentService: PaymentService, maxRetries: number = 3) {
    this.paymentService = paymentService;
    this.maxRetries = maxRetries;
  }

  /** Run a retry attempt for a failed payment. */
  run(userId: string, amount: number, card: CardDetails): PaymentResult | null {
    for (let attempt = 0; attempt < this.maxRetries; attempt++) {
      try {
        // Call processPayment — caller #5
        const result = this.paymentService.processPayment(amount, userId, card);
        if (result.status === "success") {
          return result;
        }
      } catch {
        // Retry on next iteration
        continue;
      }
    }
    return null;
  }
}
