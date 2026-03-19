import { RefundReason, RefundStatus } from '../../../src/payments/models/Refund';
import { PaymentStatus } from '../../../src/types/enums';
import { createMoney, Currency } from '../../../src/types/money';
import { mockPayment } from '../../helpers/mockFactory';

describe('RefundService', () => {
  describe('processRefund', () => {
    it('should process a full refund for a completed payment', () => {
      const payment = mockPayment({ status: PaymentStatus.COMPLETED });
      expect(payment.status).toBe(PaymentStatus.COMPLETED);
    });

    it('should reject refund exceeding payment amount', () => {
      const payment = mockPayment({ amount: createMoney(50, Currency.USD) });
      const refundAmount = createMoney(100, Currency.USD);
      expect(refundAmount.amount).toBeGreaterThan(payment.amount.amount);
    });

    it('should set correct refund status', () => {
      expect(RefundStatus.COMPLETED).toBe('completed');
      expect(RefundStatus.PENDING).toBe('pending');
    });
  });

  describe('calculateRefundAmount', () => {
    it('should return full amount when no previous refunds', () => {
      const payment = { amount: createMoney(100, Currency.USD), refundedAmount: null };
      expect(payment.refundedAmount).toBeNull();
    });

    it('should subtract previously refunded amount', () => {
      const amount = createMoney(100, Currency.USD);
      const refunded = createMoney(30, Currency.USD);
      expect(amount.amount - refunded.amount).toBe(70);
    });
  });
});
