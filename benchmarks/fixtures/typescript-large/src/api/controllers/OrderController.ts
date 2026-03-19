import { Logger } from '../../shared/utils/Logger';
import { PaymentService } from '../../payments/services/PaymentService';
import { UserService } from '../../users/services/UserService';
import { AuthGuard } from '../../auth/guards/AuthGuard';
import { ApiResponse } from '../../types/common';
import { PaymentResult } from '../../payments/types/PaymentTypes';
import { PaymentProcessor } from '../../types/enums';
import { createMoney, Currency } from '../../types/money';
import { CryptoService } from '../../shared/utils/Crypto';
import { ValidationError } from '../../types/errors';

/** Order details for checkout */
export interface CheckoutRequest {
  items: { productId: string; quantity: number; price: number }[];
  currency: Currency;
}

/** Controller for order-related endpoints */
export class OrderController {
  private paymentService: PaymentService;
  private userService: UserService;
  private authGuard: AuthGuard;
  private logger: Logger;

  constructor(paymentService: PaymentService, userService: UserService, authGuard: AuthGuard) {
    this.paymentService = paymentService;
    this.userService = userService;
    this.authGuard = authGuard;
    this.logger = new Logger('OrderController');
  }

  /**
   * POST /orders/checkout
   * Creates an order and processes payment for all items.
   * CALLS processPayment.
   */
  async checkout(authHeader: string, request: CheckoutRequest): Promise<ApiResponse<PaymentResult>> {
    const user = this.authGuard.guard(authHeader);
    const userInfo = await this.userService.getUser(user.sub);

    if (request.items.length === 0) {
      throw new ValidationError('Cart is empty');
    }

    const total = request.items.reduce((sum, item) => sum + item.price * item.quantity, 0);
    const amount = createMoney(total, request.currency);
    const crypto = new CryptoService();

    this.logger.info('Processing checkout', { userId: user.sub, total, itemCount: request.items.length });

    const result = await this.paymentService.processPayment(
      user.sub,
      amount,
      PaymentProcessor.STRIPE,
      `Order checkout: ${request.items.length} items`,
      `checkout_${user.sub}_${crypto.generateToken(8)}`,
      { items: request.items },
    );

    this.logger.info('Checkout complete', { userId: user.sub, success: result.success, paymentId: result.paymentId });

    return {
      success: result.success,
      data: result,
      message: result.success ? 'Checkout successful' : `Checkout failed: ${result.failureReason}`,
      timestamp: new Date(),
    };
  }

  /**
   * POST /orders/:orderId/retry-payment
   * Retries a failed payment for an existing order.
   * CALLS processPayment.
   */
  async retryPayment(authHeader: string, orderId: string, amount: number, currency: Currency): Promise<ApiResponse<PaymentResult>> {
    const user = this.authGuard.guard(authHeader);
    this.logger.info('Retrying payment', { userId: user.sub, orderId });

    const money = createMoney(amount, currency);
    const crypto = new CryptoService();

    const result = await this.paymentService.processPayment(
      user.sub,
      money,
      PaymentProcessor.STRIPE,
      `Retry payment for order ${orderId}`,
      `retry_${orderId}_${crypto.generateToken(8)}`,
      { orderId, retryAt: new Date().toISOString() },
    );

    return {
      success: result.success,
      data: result,
      message: result.success ? 'Payment retry successful' : `Payment retry failed: ${result.failureReason}`,
      timestamp: new Date(),
    };
  }

  /**
   * POST /orders/:orderId/cancel
   * Cancel an order (does NOT call processPayment).
   */
  async cancelOrder(authHeader: string, orderId: string): Promise<ApiResponse<{ orderId: string; cancelled: boolean }>> {
    const user = this.authGuard.guard(authHeader);
    this.logger.info('Cancelling order', { userId: user.sub, orderId });
    return {
      success: true,
      data: { orderId, cancelled: true },
      message: 'Order cancelled',
      timestamp: new Date(),
    };
  }
}
