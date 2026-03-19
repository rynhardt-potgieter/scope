import { BaseEntity, EntityId } from '../../types/common';
import { SubscriptionStatus, BillingInterval } from '../../types/enums';
import { Money } from '../../types/money';

/** A recurring subscription */
export interface Subscription extends BaseEntity {
  userId: EntityId;
  planName: string;
  status: SubscriptionStatus;
  billingInterval: BillingInterval;
  amount: Money;
  currentPeriodStart: Date;
  currentPeriodEnd: Date;
  cancelledAt: Date | null;
  cancelReason: string | null;
  trialEndsAt: Date | null;
  nextBillingDate: Date;
  failedPaymentAttempts: number;
}
