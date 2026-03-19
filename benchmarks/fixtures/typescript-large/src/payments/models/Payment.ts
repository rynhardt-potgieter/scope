import { BaseEntity, EntityId } from '../../types/common';
import { PaymentStatus } from '../../types/enums';
import { Money } from '../../types/money';

/** A payment transaction record */
export interface Payment extends BaseEntity {
  userId: EntityId;
  amount: Money;
  status: PaymentStatus;
  processor: string;
  processorTransactionId: string | null;
  description: string;
  metadata: Record<string, unknown>;
  failureReason: string | null;
  refundedAmount: Money | null;
  completedAt: Date | null;
}
