import { BaseEntity, EntityId } from '../../types/common';
import { Money } from '../../types/money';

/** Reason categories for refunds */
export enum RefundReason {
  CUSTOMER_REQUEST = 'customer_request',
  DUPLICATE_CHARGE = 'duplicate_charge',
  SERVICE_ISSUE = 'service_issue',
  FRAUD = 'fraud',
  OTHER = 'other',
}

/** Refund status */
export enum RefundStatus {
  PENDING = 'pending',
  PROCESSING = 'processing',
  COMPLETED = 'completed',
  FAILED = 'failed',
}

/** A refund against a payment */
export interface Refund extends BaseEntity {
  paymentId: EntityId;
  userId: EntityId;
  amount: Money;
  reason: RefundReason;
  status: RefundStatus;
  processorRefundId: string | null;
  notes: string;
  processedAt: Date | null;
  processedBy: EntityId | null;
}
