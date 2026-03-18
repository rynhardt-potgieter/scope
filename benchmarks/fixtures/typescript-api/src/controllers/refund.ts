import { PaymentService } from "../payments/service";
import { PaymentResult } from "../payments/types";

/** Controller handling refund-related HTTP endpoints. */
export class RefundController {
  private paymentService: PaymentService;

  constructor(paymentService: PaymentService) {
    this.paymentService = paymentService;
  }

  /** Process a refund for a given transaction. */
  processRefund(transactionId: string): PaymentResult {
    return this.paymentService.refundPayment(transactionId);
  }
}
