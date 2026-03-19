/** Request to create a subscription */
export interface CreateSubscriptionRequest {
  planName: string;
  billingInterval: string;
  amount: number;
  currency: string;
}

/** Subscription response */
export interface SubscriptionResponse {
  subscriptionId: string;
  planName: string;
  status: string;
  billingInterval: string;
  amount: number;
  currency: string;
  currentPeriodStart: string;
  currentPeriodEnd: string;
  nextBillingDate: string;
  createdAt: string;
}

/** Request to cancel a subscription */
export interface CancelSubscriptionRequest {
  reason: string;
}
