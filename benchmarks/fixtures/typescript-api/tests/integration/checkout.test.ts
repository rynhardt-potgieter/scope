// Integration test for the checkout flow

import { PaymentProcessor } from "../../src/payments/processor";
import { PaymentService } from "../../src/payments/service";
import { UserRepository } from "../../src/users/repository";
import { UserService } from "../../src/users/service";
import { OrderController } from "../../src/controllers/order";
import { CardDetails } from "../../src/payments/types";

const testCard: CardDetails = {
  cardNumber: "4111111111111111",
  expiryMonth: 12,
  expiryYear: 2030,
  cvv: "123",
  cardholderName: "Test Buyer",
};

function testFullCheckoutFlow(): void {
  const processor = new PaymentProcessor("test_key");
  const paymentService = new PaymentService(processor);
  const userRepo = new UserRepository();
  const userService = new UserService(userRepo);

  // Create a user first
  const user = userService.createUser("buyer@example.com", "Test Buyer");

  const controller = new OrderController(paymentService, userService);
  const result = controller.checkout(user.id, 99.99, testCard);

  console.assert(result.status === "success", "Checkout should succeed");
  console.assert(result.amount === 99.99, "Amount should match");
}

function testCheckoutWithInvalidUser(): void {
  const processor = new PaymentProcessor("test_key");
  const paymentService = new PaymentService(processor);
  const userRepo = new UserRepository();
  const userService = new UserService(userRepo);

  const controller = new OrderController(paymentService, userService);

  try {
    controller.checkout("nonexistent", 50, testCard);
    console.assert(false, "Should have thrown for invalid user");
  } catch (e) {
    // Expected — user not found
  }
}

// Run all tests
testFullCheckoutFlow();
testCheckoutWithInvalidUser();

console.log("All checkout integration tests passed");
