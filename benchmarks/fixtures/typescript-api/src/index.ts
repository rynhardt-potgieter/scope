import { PaymentProcessor } from "./payments/processor";
import { PaymentService } from "./payments/service";
import { CardDetails } from "./payments/types";
import { UserRepository } from "./users/repository";
import { UserService } from "./users/service";
import { NotificationService } from "./notifications/service";
import { OrderController } from "./controllers/order";
import { RefundController } from "./controllers/refund";
import { SubscriptionService } from "./controllers/subscription";
import { PaymentRetryWorker } from "./workers/payment-retry";

// Wire up dependencies
const processor = new PaymentProcessor("sk_test_api_key");
const paymentService = new PaymentService(processor);

const userRepository = new UserRepository();
const userService = new UserService(userRepository);

const notificationService = new NotificationService(
  "https://email.example.com",
  "https://sms.example.com"
);

// Controllers and workers
const orderController = new OrderController(paymentService, userService);
const refundController = new RefundController(paymentService);
const subscriptionService = new SubscriptionService(paymentService);
const paymentRetryWorker = new PaymentRetryWorker(paymentService);

/** Run a zero-amount validation charge on startup to verify gateway connectivity. */
function verifyPaymentGateway(service: PaymentService): void {
  const testCard: CardDetails = {
    cardNumber: "4000000000000000",
    expiryMonth: 12,
    expiryYear: 2099,
    cvv: "000",
    cardholderName: "Gateway Test",
  };
  // Call processPayment — caller #7 (startup validation)
  service.processPayment(0, "system", testCard);
}

export {
  paymentService,
  userService,
  notificationService,
  orderController,
  refundController,
  subscriptionService,
  paymentRetryWorker,
  verifyPaymentGateway,
};
