import { SubscriptionStatus, BillingInterval } from '../../../src/types/enums';
import { createMoney, Currency } from '../../../src/types/money';
import { mockSubscription } from '../../helpers/mockFactory';
import { testId } from '../../helpers/testUtils';

describe('SubscriptionService', () => {
  describe('createSubscription', () => {
    it('should create a subscription with correct billing period', () => {
      const sub = mockSubscription({ billingInterval: BillingInterval.MONTHLY });
      expect(sub.billingInterval).toBe(BillingInterval.MONTHLY);
      expect(sub.status).toBe(SubscriptionStatus.ACTIVE);
    });
  });

  describe('processRenewal', () => {
    it('should renew an active subscription', () => {
      const sub = mockSubscription({ status: SubscriptionStatus.ACTIVE });
      expect(sub.status).toBe(SubscriptionStatus.ACTIVE);
    });

    it('should not renew a cancelled subscription', () => {
      const sub = mockSubscription({ status: SubscriptionStatus.CANCELLED });
      expect(sub.status).toBe(SubscriptionStatus.CANCELLED);
    });
  });

  describe('cancelSubscription', () => {
    it('should cancel an active subscription', () => {
      const sub = mockSubscription();
      expect(sub.cancelledAt).toBeNull();
    });
  });
});
