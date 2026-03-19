import { Subscription } from '../models/Subscription';

/** Subscription response DTO */
export interface SubscriptionDto {
  id: string;
  userId: string;
  planName: string;
  status: string;
  billingInterval: string;
  amount: number;
  currency: string;
  currentPeriodStart: string;
  currentPeriodEnd: string;
  nextBillingDate: string;
  cancelledAt: string | null;
  createdAt: string;
}

/** Maps Subscription entities to DTOs */
export class SubscriptionMapper {
  /** Convert a Subscription entity to a response DTO */
  static toDto(subscription: Subscription): SubscriptionDto {
    return {
      id: subscription.id,
      userId: subscription.userId,
      planName: subscription.planName,
      status: subscription.status,
      billingInterval: subscription.billingInterval,
      amount: subscription.amount.amount,
      currency: subscription.amount.currency,
      currentPeriodStart: subscription.currentPeriodStart.toISOString(),
      currentPeriodEnd: subscription.currentPeriodEnd.toISOString(),
      nextBillingDate: subscription.nextBillingDate.toISOString(),
      cancelledAt: subscription.cancelledAt?.toISOString() ?? null,
      createdAt: subscription.createdAt.toISOString(),
    };
  }

  /** Convert a list of Subscriptions to DTOs */
  static toDtoList(subscriptions: Subscription[]): SubscriptionDto[] {
    return subscriptions.map(SubscriptionMapper.toDto);
  }
}
