import { PaymentService } from '../payments/service';

export class RefundController {
  private paymentService: PaymentService;

  constructor(paymentService: PaymentService) {
    this.paymentService = paymentService;
  }

  async processRefund(transactionId: string): Promise<boolean> {
    return await this.paymentService.refundPayment(transactionId);
  }
}
