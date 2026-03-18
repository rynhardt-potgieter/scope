export interface PaymentRequest {
  amount: number;
  userId: string;
  currency?: string;
}

export interface PaymentResult {
  success: boolean;
  transactionId: string;
  error?: string;
}

export type PaymentStatus = 'pending' | 'completed' | 'failed';
