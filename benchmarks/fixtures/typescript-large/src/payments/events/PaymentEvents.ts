import { DomainEvent, EventType, EventMetadata } from '../../types/events';
import { EntityId } from '../../types/common';
import { Money } from '../../types/money';

/** Create a PaymentProcessed event */
export function createPaymentProcessedEvent(
  paymentId: EntityId,
  userId: EntityId,
  amount: Money,
  metadata: EventMetadata,
): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.PAYMENT_PROCESSED,
    timestamp: new Date(),
    aggregateId: paymentId,
    aggregateType: 'Payment',
    payload: { paymentId, userId, amount: amount.amount, currency: amount.currency },
    metadata,
  };
}

/** Create a PaymentRefunded event */
export function createPaymentRefundedEvent(
  paymentId: EntityId,
  refundAmount: Money,
  metadata: EventMetadata,
): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.PAYMENT_REFUNDED,
    timestamp: new Date(),
    aggregateId: paymentId,
    aggregateType: 'Payment',
    payload: { paymentId, refundAmount: refundAmount.amount, currency: refundAmount.currency },
    metadata,
  };
}

/** Create a SubscriptionRenewed event */
export function createSubscriptionRenewedEvent(
  subscriptionId: EntityId,
  userId: EntityId,
  metadata: EventMetadata,
): DomainEvent {
  return {
    id: `evt_${Date.now()}`,
    type: EventType.SUBSCRIPTION_RENEWED,
    timestamp: new Date(),
    aggregateId: subscriptionId,
    aggregateType: 'Subscription',
    payload: { subscriptionId, userId },
    metadata,
  };
}
