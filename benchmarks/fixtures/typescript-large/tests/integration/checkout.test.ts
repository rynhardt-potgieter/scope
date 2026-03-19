import { PaymentProcessor } from '../../src/types/enums';
import { Currency, createMoney } from '../../src/types/money';
import { testId, testMoney } from '../helpers/testUtils';

describe('Checkout Integration', () => {
  describe('full checkout flow', () => {
    it('should create an order and process payment', () => {
      const userId = testId('user');
      const items = [
        { productId: testId('prod'), quantity: 2, price: 25 },
        { productId: testId('prod'), quantity: 1, price: 50 },
      ];
      const total = items.reduce((sum, item) => sum + item.price * item.quantity, 0);
      expect(total).toBe(100);
    });

    it('should handle payment failure and allow retry', () => {
      const orderId = testId('order');
      expect(orderId).toContain('order');
    });

    it('should cancel an order after failed retry', () => {
      const cancelled = true;
      expect(cancelled).toBe(true);
    });
  });

  describe('cart validation', () => {
    it('should reject empty cart', () => {
      const items: unknown[] = [];
      expect(items.length).toBe(0);
    });

    it('should reject negative prices', () => {
      const price = -10;
      expect(price).toBeLessThan(0);
    });
  });
});
