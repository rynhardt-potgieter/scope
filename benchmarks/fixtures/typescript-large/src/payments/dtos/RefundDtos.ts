/** Request to create a refund */
export interface CreateRefundRequest {
  paymentId: string;
  amount: number;
  currency: string;
  reason: string;
  notes?: string;
}

/** Refund response */
export interface RefundResponse {
  refundId: string;
  paymentId: string;
  amount: number;
  currency: string;
  status: string;
  processedAt: string | null;
}

/** Partial refund with replacement request */
export interface PartialRefundWithReplacementRequest {
  originalPaymentId: string;
  refundAmount: number;
  replacementAmount: number;
  currency: string;
  reason: string;
}
