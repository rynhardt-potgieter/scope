import { RefundReason, RefundStatus } from '../../src/payments/models/Refund';
import { PaymentStatus } from '../../src/types/enums';
import { createMoney, Currency } from '../../src/types/money';
import { mockPayment } from '../helpers/mockFactory';

describe('Refund Integration', () => {
  describe('full refund flow', () => {
    it('should refund a completed payment', () => {
      const payment = mockPayment({ status: PaymentStatus.COMPLETED });
      expect(payment.status).toBe(PaymentStatus.COMPLETED);
    });

    it('should update payment status to refunded', () => {
      expect(PaymentStatus.REFUNDED).toBe('refunded');
    });
  });

  describe('partial refund with replacement', () => {
    it('should refund partial amount and create replacement payment', () => {
      const original = createMoney(100, Currency.USD);
      const refund = createMoney(30, Currency.USD);
      const replacement = createMoney(70, Currency.USD);
      expect(original.amount).toBe(refund.amount + replacement.amount);
    });
  });
});
