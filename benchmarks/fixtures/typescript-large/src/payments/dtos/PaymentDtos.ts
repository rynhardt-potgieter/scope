/** Request to create a new payment */
export interface CreatePaymentRequest {
  amount: number;
  currency: string;
  processor: string;
  description: string;
  cardToken: string;
  idempotencyKey: string;
  metadata?: Record<string, unknown>;
}

/** Payment response */
export interface PaymentResponse {
  paymentId: string;
  status: string;
  processorTransactionId: string | null;
  amount: number;
  currency: string;
  createdAt: string;
}

/** Payment list query parameters */
export interface ListPaymentsQuery {
  userId?: string;
  status?: string;
  startDate?: string;
  endDate?: string;
  page?: number;
  pageSize?: number;
}
