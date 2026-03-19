import { EntityId } from '../../src/types/common';
import { Currency, createMoney, Money } from '../../src/types/money';
import { UserRole, PaymentProcessor } from '../../src/types/enums';
import { PaymentRequest } from '../../src/payments/types/PaymentTypes';

/** Generate a unique test ID */
export function testId(prefix: string = 'test'): EntityId {
  return `${prefix}_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

/** Create test money value */
export function testMoney(amount: number = 99.99, currency: Currency = Currency.USD): Money {
  return createMoney(amount, currency);
}

/** Create a test payment request */
export function testPaymentRequest(overrides?: Partial<PaymentRequest>): PaymentRequest {
  return {
    userId: testId('user'),
    amount: testMoney(),
    processor: PaymentProcessor.STRIPE,
    description: 'Test payment',
    idempotencyKey: testId('idem'),
    ...overrides,
  };
}

/** Sleep for a specified number of milliseconds */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Assert that an async function throws a specific error type */
export async function expectError<T extends Error>(
  fn: () => Promise<unknown>,
  errorType: new (...args: any[]) => T,
): Promise<T> {
  try {
    await fn();
    throw new Error(`Expected ${errorType.name} but no error was thrown`);
  } catch (error) {
    if (error instanceof errorType) return error;
    throw error;
  }
}
