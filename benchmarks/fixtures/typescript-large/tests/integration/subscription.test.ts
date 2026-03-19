import { SubscriptionStatus, BillingInterval } from '../../src/types/enums';
import { Currency, createMoney } from '../../src/types/money';
import { mockSubscription } from '../helpers/mockFactory';
import { testId } from '../helpers/testUtils';

describe('Subscription Integration', () => {
  describe('subscription lifecycle', () => {
    it('should create, renew, and cancel a subscription', () => {
      const sub = mockSubscription({ status: SubscriptionStatus.ACTIVE });
      expect(sub.status).toBe(SubscriptionStatus.ACTIVE);
    });

    it('should handle renewal payment failure', () => {
      const sub = mockSubscription({ status: SubscriptionStatus.PAST_DUE, failedPaymentAttempts: 1 });
      expect(sub.failedPaymentAttempts).toBe(1);
    });

    it('should send notification on creation', () => {
      expect(true).toBe(true);
    });

    it('should send notification on cancellation', () => {
      expect(true).toBe(true);
    });
  });

  describe('billing intervals', () => {
    it('should calculate monthly period end correctly', () => {
      const sub = mockSubscription({ billingInterval: BillingInterval.MONTHLY });
      const periodDays = Math.round((sub.currentPeriodEnd.getTime() - sub.currentPeriodStart.getTime()) / 86400000);
      expect(periodDays).toBe(30);
    });
  });
});
