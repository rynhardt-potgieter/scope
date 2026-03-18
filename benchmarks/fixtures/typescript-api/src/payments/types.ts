/** Details for a credit/debit card payment method. */
export interface CardDetails {
  cardNumber: string;
  expiryMonth: number;
  expiryYear: number;
  cvv: string;
  cardholderName: string;
}

/** Request payload for processing a payment. */
export interface PaymentRequest {
  userId: string;
  amount: number;
  currency: string;
  card: CardDetails;
  description?: string;
}

/** Result of a payment processing operation. */
export interface PaymentResult {
  transactionId: string;
  status: "success" | "failed" | "pending";
  amount: number;
  currency: string;
  timestamp: Date;
  errorMessage?: string;
}
