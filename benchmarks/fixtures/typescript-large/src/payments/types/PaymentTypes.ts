import { Money } from '../../types/money';
import { EntityId } from '../../types/common';
import { PaymentProcessor } from '../../types/enums';

/** Inbound payment request */
export interface PaymentRequest {
  userId: EntityId;
  amount: Money;
  processor: PaymentProcessor;
  description: string;
  idempotencyKey: string;
  metadata?: Record<string, unknown>;
}

/** Result returned by the payment processing pipeline */
export interface PaymentResult {
  success: boolean;
  paymentId: EntityId;
  processorTransactionId: string | null;
  status: string;
  failureReason: string | null;
}

/** Credit card details (tokenized; never store raw PAN) */
export interface CardDetails {
  token: string;
  last4: string;
  brand: string;
  expiryMonth: number;
  expiryYear: number;
}

/** Processor charge request */
export interface ChargeRequest {
  amount: Money;
  cardToken: string;
  description: string;
  idempotencyKey: string;
  metadata?: Record<string, unknown>;
}

/** Processor charge response */
export interface ChargeResponse {
  success: boolean;
  transactionId: string;
  failureReason: string | null;
}
