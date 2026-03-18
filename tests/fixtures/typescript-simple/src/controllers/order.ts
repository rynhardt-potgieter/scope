import { PaymentService } from '../payments/service';
import { PaymentRequest } from '../payments/types';

export class OrderController {
  private paymentService: PaymentService;

  constructor(paymentService: PaymentService) {
    this.paymentService = paymentService;
  }

  async checkout(amount: number, userId: string): Promise<void> {
    const request: PaymentRequest = { amount, userId };
    await this.paymentService.processPayment(request);
  }

  async retryPayment(amount: number, userId: string): Promise<void> {
    const request: PaymentRequest = { amount, userId };
    await this.paymentService.processPayment(request);
  }
}
