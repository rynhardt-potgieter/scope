import { PaymentService } from "../payments/service";
import { CardDetails, PaymentResult } from "../payments/types";
import { UserService } from "../users/service";

/** Controller handling order-related HTTP endpoints. */
export class OrderController {
  private paymentService: PaymentService;
  private userService: UserService;

  constructor(paymentService: PaymentService, userService: UserService) {
    this.paymentService = paymentService;
    this.userService = userService;
  }

  /** Process a checkout for the given user and cart total. */
  checkout(userId: string, amount: number, card: CardDetails): PaymentResult {
    const user = this.userService.getUser(userId);
    // Call processPayment — caller #1
    const result = this.paymentService.processPayment(amount, user.id, card);
    return result;
  }

  /** Retry a failed payment for an existing order. */
  retryPayment(userId: string, amount: number, card: CardDetails): PaymentResult {
    // Call processPayment — caller #2
    const result = this.paymentService.processPayment(amount, userId, card);
    return result;
  }

  /** Process a split payment — pays part now and schedules the rest. */
  splitPayment(userId: string, totalAmount: number, splitRatio: number, card: CardDetails): PaymentResult {
    const firstAmount = totalAmount * splitRatio;
    // Call processPayment — caller #3
    const result = this.paymentService.processPayment(firstAmount, userId, card);
    return result;
  }

  /** Cancel an order and refund the payment. */
  cancelOrder(transactionId: string): PaymentResult {
    return this.paymentService.refundPayment(transactionId);
  }
}
