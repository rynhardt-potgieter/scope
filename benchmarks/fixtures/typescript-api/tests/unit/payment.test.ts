// Unit tests for PaymentService
// In a real fixture these would use a test runner like Jest.
// For the benchmark fixture, compilation is the primary check.

import { PaymentService } from "../../src/payments/service";
import { PaymentProcessor } from "../../src/payments/processor";
import { CardDetails } from "../../src/payments/types";

const testCard: CardDetails = {
  cardNumber: "4111111111111111",
  expiryMonth: 12,
  expiryYear: 2030,
  cvv: "123",
  cardholderName: "Test User",
};

function testProcessPaymentSuccess(): void {
  const processor = new PaymentProcessor("test_key");
  const service = new PaymentService(processor);
  const result = service.processPayment(100, "user_1", testCard);
  console.assert(result.status === "success", "Expected success status");
  console.assert(result.amount === 100, "Expected amount 100");
}

function testProcessPaymentStoresTransaction(): void {
  const processor = new PaymentProcessor("test_key");
  const service = new PaymentService(processor);
  const result = service.processPayment(50, "user_1", testCard);
  const stored = service.getTransaction(result.transactionId);
  console.assert(stored !== undefined, "Transaction should be stored");
}

function testRefundPaymentSuccess(): void {
  const processor = new PaymentProcessor("test_key");
  const service = new PaymentService(processor);
  const payment = service.processPayment(75, "user_1", testCard);
  const refund = service.refundPayment(payment.transactionId);
  console.assert(refund.status === "success", "Expected refund success");
}

function testRefundPaymentNotFound(): void {
  const processor = new PaymentProcessor("test_key");
  const service = new PaymentService(processor);
  const result = service.refundPayment("nonexistent");
  console.assert(result.status === "failed", "Expected failed status");
}

function testValidateCardRejectsShortNumber(): void {
  const processor = new PaymentProcessor("test_key");
  const service = new PaymentService(processor);
  const badCard: CardDetails = { ...testCard, cardNumber: "123" };
  try {
    service.validateCard(badCard);
    console.assert(false, "Should have thrown");
  } catch (e) {
    // Expected
  }
}

function testValidateCardRejectsExpired(): void {
  const processor = new PaymentProcessor("test_key");
  const service = new PaymentService(processor);
  const expiredCard: CardDetails = { ...testCard, expiryYear: 2020 };
  try {
    service.validateCard(expiredCard);
    console.assert(false, "Should have thrown");
  } catch (e) {
    // Expected
  }
}

function testGetTransactionReturnsUndefinedForMissing(): void {
  const processor = new PaymentProcessor("test_key");
  const service = new PaymentService(processor);
  const result = service.getTransaction("missing");
  console.assert(result === undefined, "Expected undefined");
}

function testProcessPaymentValidatesCard(): void {
  const processor = new PaymentProcessor("test_key");
  const service = new PaymentService(processor);
  const badCard: CardDetails = { ...testCard, cvv: "1" };
  try {
    service.processPayment(100, "user_1", badCard);
    console.assert(false, "Should have thrown for bad CVV");
  } catch (e) {
    // Expected
  }
}

// Run all tests
testProcessPaymentSuccess();
testProcessPaymentStoresTransaction();
testRefundPaymentSuccess();
testRefundPaymentNotFound();
testValidateCardRejectsShortNumber();
testValidateCardRejectsExpired();
testGetTransactionReturnsUndefinedForMissing();
testProcessPaymentValidatesCard();

console.log("All payment tests passed");
